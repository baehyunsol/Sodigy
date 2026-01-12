use sodigy::{
    CliCommand,
    Command,
    CompileStage,
    COMPILE_STAGES,
    EmitIrOption,
    Error,
    ModuleCompileState,
    Profile,
    StoreIrAt,
    parse_args,
};
use sodigy_code_gen::Backend;
use sodigy_endec::{DumpSession, Endec};
use sodigy_error::{Error as SodigyError, Warning as SodigyWarning};
use sodigy_file::{File, FileOrStd, ModulePath};
use sodigy_fs_api::{
    FileError,
    FileErrorKind,
    WriteMode,
    basename,
    create_dir,
    create_dir_all,
    exists,
    join,
    join3,
    join4,
    parent,
    read_bytes,
    read_dir,
    set_current_dir,
    write_bytes,
    write_string,
};
use sodigy_hir as hir;
use sodigy_mir as mir;
use sodigy_optimize::OptimizeLevel;
use sodigy_span::Span;
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Instant;

mod worker;

use worker::{Channel, MessageToMain, MessageToWorker, init_workers};

fn main() {
    let result = run();

    match result {
        Ok(()) => {},
        Err(e) => {
            match &e {
                Error::RuntimeError => {
                    // TODO: what do I do here?
                },
                Error::CompileError => {
                    // The errors are already dumped!
                },
                Error::FileError(e) => {
                    eprintln!("FileError: {e:?}");
                },
                Error::DecodeError(e) => {
                    eprintln!("DecodeError: {e:?}");
                },
                Error::CliError(e) => {
                    let message = e.kind.render();
                    eprintln!(
                        "cli error: {message}{}",
                        if let Some(span) = &e.span {
                            format!("\n\n{}", ragit_cli::underline_span(span))
                        } else {
                            String::new()
                        },
                    );
                },
                Error::MpscError => {
                    eprintln!("MpscError");
                },
                Error::IrCacheNotFound(s) => {
                    eprintln!("IrCacheNotFound({s:?})");
                },
                Error::MiscError => {
                    eprintln!("Unknown Error");
                },
            }

            std::process::exit(e.exit_code())
        },
    }
}


fn run() -> Result<(), Error> {
    let args = std::env::args().collect::<Vec<_>>();

    // TODO: make it configurable
    let ir_dir = String::from("target");

    match parse_args(&args)? {
        CliCommand::New { project_name } => {
            init_project(&project_name)?;
            Ok(())
        },
        cli_command @ (
            CliCommand::Build { optimize_level, import_std, emit_irs, jobs, .. } |
            CliCommand::Run { optimize_level, import_std, emit_irs, jobs } |
            CliCommand::Test { optimize_level, import_std, emit_irs, jobs }
        ) => {
            let started_at = Instant::now();
            let mut errors = vec![];
            let mut warnings = vec![];
            let workers = init_workers(jobs);
            let (output_path, backend) = match &cli_command {
                CliCommand::Run { .. } => (StoreIrAt::IntermediateDir, Backend::Bytecode),
                CliCommand::Test { .. } => (StoreIrAt::IntermediateDir, Backend::Bytecode),
                CliCommand::Build { output_path, backend, .. } => (StoreIrAt::File(output_path.to_string()), *backend),
                _ => todo!(),
            };

            let result = compile(
                output_path,
                backend,
                ir_dir.clone(),
                optimize_level,
                import_std,
                emit_irs,
                &workers,
                &mut errors,
                &mut warnings,
            );

            let elapsed_ms = Instant::now().duration_since(started_at).as_millis();
            sodigy_error::dump_errors(
                errors,
                warnings,
                &ir_dir,
                sodigy_error::DumpErrorOption::default(),
                Some(elapsed_ms as u64),
            );

            for worker in workers.iter() {
                let _ = worker.send(MessageToWorker::Kill);
            }

            for worker in workers.into_iter() {
                let _ = worker.join();
            }

            result?;

            match cli_command {
                CliCommand::Run { .. } => interpret(StoreIrAt::IntermediateDir, Profile::Script, &ir_dir),
                CliCommand::Test { .. } => interpret(StoreIrAt::IntermediateDir, Profile::Test, &ir_dir),
                _ => Ok(()),
            }
        },
        _ => todo!(),
    }
}

// How it handles compile errors/warnings:
// 1. When a worker finishes a stage, the worker sends all the errors/warnings to the master.
//   - The worker doesn't discard the errors/warnings. Errors/warnings are never discarded.
//     Even endec keeps all the errors/warnings. Errors/warnings generated in inter-hir stage
//     are propagated to all the mir sessions, so there can be duplicate errors/warnings.
// 2. If the master receives an error, it immediately halts the compilation and dumps the errors/warnings.
//   - There can be duplicate errors/warnings, so the master is responsible for deduplication.

fn compile(
    output_path: StoreIrAt,
    backend: Backend,
    ir_dir: String,
    optimize_level: OptimizeLevel,
    import_std: bool,
    emit_irs: bool,
    workers: &[Channel],
    errors: &mut Vec<SodigyError>,
    warnings: &mut Vec<SodigyWarning>,
) -> Result<(), Error> {
    goto_root_dir()?;
    let mut round_robin = 0;
    let mut modules: HashMap<ModulePath, ModuleCompileState> = HashMap::new();
    let emit_irs = if emit_irs {
        [
            CompileStage::Lex,
            CompileStage::Parse,
            CompileStage::Hir,
            CompileStage::InterHir,
            CompileStage::Mir,
            CompileStage::InterMir,
            CompileStage::PostMir,
            CompileStage::Bytecode,
        ].into_iter().map(
            |stage| EmitIrOption {
                stage,
                store: StoreIrAt::IntermediateDir,
                human_readable: true,
            }
        ).collect()
    } else {
        vec![]
    };

    let lib_module_path = ModulePath::lib();
    let lib_file_path = match lib_module_path.get_file_path() {
        Ok(p) => p,
        Err(e) => {
            errors.push(SodigyError {
                kind: e.into(),
                spans: Span::Lib.simple_error(),
                note: None,
            });
            return Err(Error::CompileError);
        },
    };
    modules.insert(lib_module_path.clone(), ModuleCompileState {
        module_path: lib_module_path,
        file_path: lib_file_path,
        span: Span::Lib,
        compile_stage: CompileStage::Load,
        running: false,
    });
    init_ir_dir(&ir_dir)?;

    if import_std {
        let (std_module_path, std_file_path) = sodigy_file::std_root();
        modules.insert(
            std_module_path.clone(),
            ModuleCompileState {
                module_path: std_module_path,
                file_path: std_file_path,
                span: Span::Std,
                compile_stage: CompileStage::Load,
                running: false,
            },
        );
    }

    loop {
        let mut every_hir_complete = true;
        let mut every_mir_complete = true;
        let mut every_post_mir_complete = true;

        for module in modules.values_mut() {
            if let (CompileStage::Load, false) = (module.compile_stage, module.running) {
                workers[round_robin % workers.len()].send(MessageToWorker::Run(vec![
                    Command::PerFileIr {
                        input_file_path: module.file_path.clone(),
                        input_module_path: module.module_path.clone(),
                        intermediate_dir: ir_dir.clone(),
                        find_modules: true,
                        emit_ir_options: emit_irs.clone_and_push(
                            EmitIrOption {
                                stage: CompileStage::Hir,
                                store: StoreIrAt::IntermediateDir,
                                human_readable: false,
                            },
                        ),
                        stop_after: CompileStage::Hir,
                    },
                ]))?;
                round_robin += 1;
                module.compile_stage = CompileStage::Hir;
                module.running = true;
            }

            if (module.compile_stage, module.running) != (CompileStage::Hir, false) {
                every_hir_complete = false;
            }

            if (module.compile_stage, module.running) != (CompileStage::Mir, false) {
                every_mir_complete = false;
            }

            if (module.compile_stage, module.running) != (CompileStage::PostMir, false) {
                every_post_mir_complete = false;
            }
        }

        if every_hir_complete {
            workers[round_robin % workers.len()].send(MessageToWorker::Run(vec![
                Command::InterHir {
                    modules: modules.values().map(
                        |module| (module.module_path.clone(), module.span)
                    ).collect(),
                    intermediate_dir: ir_dir.clone(),
                    emit_ir_options: emit_irs.clone_and_push(
                        EmitIrOption {
                            stage: CompileStage::InterHir,
                            store: StoreIrAt::IntermediateDir,
                            human_readable: false,
                        },
                    ),
                },
            ]))?;
            round_robin += 1;

            for module in modules.values_mut() {
                module.compile_stage = CompileStage::InterHir;
                module.running = true;
            }
        }

        if every_mir_complete {
            workers[round_robin % workers.len()].send(MessageToWorker::Run(vec![
                Command::InterMir {
                    modules: modules.values().map(
                        |module| (module.module_path.clone(), module.span)
                    ).collect(),
                    intermediate_dir: ir_dir.clone(),
                    emit_ir_options: emit_irs.clone_and_push(
                        EmitIrOption {
                            stage: CompileStage::InterMir,
                            store: StoreIrAt::IntermediateDir,
                            human_readable: false,
                        },
                    ),
                },
            ]))?;
            round_robin += 1;

            for module in modules.values_mut() {
                module.compile_stage = CompileStage::InterMir;
                module.running = true;
            }
        }

        if every_post_mir_complete {
            workers[round_robin % workers.len()].send(MessageToWorker::Run(vec![
                Command::Bytecode {
                    modules: modules.values().map(
                        |module| (module.module_path.clone(), module.span)
                    ).collect(),
                    intermediate_dir: ir_dir.clone(),
                    optimize_level,
                    backend,
                    output_path: output_path.clone(),
                    stop_after: CompileStage::CodeGen,
                }],
            ))?;
            round_robin += 1;

            for module in modules.values_mut() {
                module.compile_stage = CompileStage::CodeGen;
                module.running = true;
            }
        }

        for worker in workers.iter() {
            match worker.try_recv() {
                Ok(msg) => match msg {
                    MessageToMain::AddModule { path, span } => {
                        if !modules.contains_key(&path) {
                            let file_path = match path.get_file_path() {
                                Ok(p) => p,
                                Err(e) => {
                                    errors.push(SodigyError {
                                        kind: e.into(),
                                        spans: span.simple_error(),
                                        note: None,
                                    });
                                    return Err(Error::CompileError);
                                },
                            };
                            modules.insert(
                                path.clone(),
                                ModuleCompileState {
                                    module_path: path,
                                    file_path,
                                    span,
                                    compile_stage: CompileStage::Load,
                                    running: false,
                                },
                            );
                        }
                    },
                    MessageToMain::IrComplete {
                        module_path,
                        compile_stage,
                        errors: errors_,
                        warnings: warnings_,
                    } => {
                        errors.extend(errors_);
                        warnings.extend(warnings_);
                        *errors = sodigy_error::deduplicate(errors);
                        *warnings = sodigy_error::deduplicate(warnings);

                        if !errors.is_empty() {
                            return Err(Error::CompileError);
                        }

                        match (compile_stage, module_path) {
                            (CompileStage::InterHir, None) => {
                                for module in modules.values_mut() {
                                    module.compile_stage = CompileStage::InterHir;
                                    module.running = false;

                                    workers[round_robin % workers.len()].send(MessageToWorker::Run(vec![
                                        Command::PerFileIr {
                                            input_file_path: module.file_path.clone(),
                                            input_module_path: module.module_path.clone(),
                                            intermediate_dir: ir_dir.clone(),
                                            find_modules: false,
                                            emit_ir_options: emit_irs.clone_and_push(
                                                EmitIrOption {
                                                    stage: CompileStage::Mir,
                                                    store: StoreIrAt::IntermediateDir,
                                                    human_readable: false,
                                                },
                                            ),
                                            stop_after: CompileStage::Mir,
                                        },
                                    ]))?;
                                    round_robin += 1;
                                }
                            },
                            (CompileStage::InterMir, None) => {
                                for module in modules.values_mut() {
                                    module.compile_stage = CompileStage::InterMir;
                                    module.running = false;

                                    workers[round_robin % workers.len()].send(MessageToWorker::Run(vec![
                                        Command::PerFileIr {
                                            input_file_path: module.file_path.clone(),
                                            input_module_path: module.module_path.clone(),
                                            intermediate_dir: ir_dir.clone(),
                                            find_modules: false,
                                            emit_ir_options: emit_irs.clone_and_push(
                                                EmitIrOption {
                                                    stage: CompileStage::PostMir,
                                                    store: StoreIrAt::IntermediateDir,
                                                    human_readable: false,
                                                },
                                            ),
                                            stop_after: CompileStage::PostMir,
                                        }],
                                    ))?;
                                    round_robin += 1;
                                }
                            },
                            // Everything is complete!
                            (CompileStage::CodeGen, None) => {
                                return Ok(());
                            },
                            (_, Some(module_path)) => {
                                match modules.get_mut(&module_path) {
                                    Some(module) => {
                                        module.compile_stage = compile_stage;
                                        module.running = false;
                                    },
                                    None => unreachable!(),
                                }
                            },
                            _ => unreachable!(),
                        }
                    },
                    MessageToMain::Error(e) => {
                        return Err(e);
                    },
                },
                Err(mpsc::TryRecvError::Empty) => {},
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(Error::MpscError);
                },
            }
        }
    }
}

fn interpret(exe: StoreIrAt, profile: Profile, intermediate_dir: &str) -> Result<(), Error> {
    let exe_bytes = match exe {
        StoreIrAt::File(f) => read_bytes(&f)?,
        StoreIrAt::IntermediateDir => get_cached_ir(
            intermediate_dir,
            CompileStage::CodeGen,
            None,
        )?.ok_or(Error::IrCacheNotFound(CompileStage::CodeGen))?,
    };

    // `emit_irs_if_has_to` will encode `Vec<u8>` twice...
    let exe_bytes = Vec::<u8>::decode(&exe_bytes)?;
    let exe = sodigy_bytecode::Executable::decode(&exe_bytes)?;

    match profile {
        // TODO: I want to spawn a separate process for assertions and capture stderr/stdout
        //       of the process, but that's extremely difficult.
        Profile::Test => {
            let mut ever_failed = false;

            for (name, label) in exe.asserts.iter() {
                let fail = sodigy_interpreter::interpret(&exe, *label).is_err();
                println!("assertion `{name}`: {}", if fail { "fail" } else { "success" });

                if fail {
                    ever_failed = true;
                }
            }

            if ever_failed {
                return Err(Error::RuntimeError);
            }
        },
        Profile::Script => todo!(),
    }

    Ok(())
}

fn goto_root_dir() -> Result<(), FileError> {
    // In some os, running `set_current_dir("..")` at `/` is nop.
    for _ in 0..64 {
        for f in read_dir(".", false)? {
            if basename(&f)? == "sodigy.toml" {
                return Ok(());
            }
        }

        set_current_dir("..")?;
    }

    Err(FileError {
        kind: FileErrorKind::FileNotFound,
        given_path: Some(String::from("sodigy.toml")),
    })
}

fn init_project(name: &str) -> Result<(), FileError> {
    // TODO: make sure that `project_name` is a valid identifier

    if exists(&name) {
        eprintln!("`{name}` already exists!");
        return Err(FileError {
            kind: FileErrorKind::AlreadyExists,
            given_path: Some(name.to_string()),
        });
    }

    create_dir(&name)?;
    let src = join(&name, "src")?;
    create_dir(&src)?;

    let lib = join(&src, "lib.sdg")?;
    write_string(&lib, "", WriteMode::CreateOrTruncate)?;

    let config = join(&name, "sodigy.toml")?;
    write_string(
        &config,
        "# TODO",
        WriteMode::CreateOrTruncate,
    )?;
    Ok(())
}

pub fn run_worker(
    commands: Vec<Command>,
    tx_to_main: mpsc::Sender<MessageToMain>,
) -> Result<(), Error> {
    for command in commands.into_iter() {
        match command {
            Command::PerFileIr {
                input_file_path,
                input_module_path,
                intermediate_dir,
                find_modules,
                emit_ir_options,
                stop_after,
            } => {
                let (is_std, file) = match &input_file_path {
                    FileOrStd::File(path) => (
                        false,
                        File::register(
                            0,  // project_id
                            &path,
                            &input_module_path.to_string(),
                            &intermediate_dir,
                        )?,
                    ),
                    FileOrStd::Std(n) => (true, File::Std(*n)),
                };
                let content_hash = file.get_content_hash(&intermediate_dir)?;

                let mut mir_session = if stop_after >= CompileStage::Mir && let Some(mir_session_bytes) = get_cached_ir(
                    &intermediate_dir,
                    CompileStage::Mir,
                    Some(content_hash),
                )? {
                    let mut s = mir::Session::decode(&mir_session_bytes)?;
                    s.intermediate_dir = intermediate_dir.clone();
                    s
                } else {
                    let mut hir_session = if let Some(hir_session_bytes) = get_cached_ir(
                        &intermediate_dir,
                        CompileStage::Hir,
                        Some(content_hash),
                    )? {
                        let mut s = hir::Session::decode(&hir_session_bytes)?;
                        s.intermediate_dir = intermediate_dir.clone();
                        s
                    } else {
                        let bytes = file.read_bytes(&intermediate_dir)?.ok_or(Error::MiscError)?;

                        let lex_session = sodigy_lex::lex(
                            file,
                            bytes,
                            intermediate_dir.clone(),
                            is_std,
                        );
                        emit_irs_if_has_to(
                            &lex_session,
                            &emit_ir_options,
                            CompileStage::Lex,
                            Some(content_hash),
                            &intermediate_dir,
                        )?;

                        if !lex_session.errors.is_empty() || stop_after <= CompileStage::Lex {
                            tx_to_main.send(MessageToMain::IrComplete {
                                module_path: Some(input_module_path),
                                compile_stage: CompileStage::Lex,
                                errors: lex_session.errors.clone(),
                                warnings: lex_session.warnings.clone(),
                            })?;

                            if !lex_session.errors.is_empty() {
                                return Err(Error::CompileError);
                            }

                            else {
                                continue;
                            }
                        }

                        let parse_session = sodigy_parse::parse(lex_session);
                        emit_irs_if_has_to(
                            &parse_session,
                            &emit_ir_options,
                            CompileStage::Parse,
                            Some(content_hash),
                            &intermediate_dir,
                        )?;

                        if !parse_session.errors.is_empty() || stop_after <= CompileStage::Parse {
                            tx_to_main.send(MessageToMain::IrComplete {
                                module_path: Some(input_module_path),
                                compile_stage: CompileStage::Parse,
                                errors: parse_session.errors.clone(),
                                warnings: parse_session.warnings.clone(),
                            })?;

                            if !parse_session.errors.is_empty() {
                                return Err(Error::CompileError);
                            }

                            else {
                                continue;
                            }
                        }

                        let hir_session = sodigy_hir::lower(parse_session);
                        emit_irs_if_has_to(
                            &hir_session,
                            &emit_ir_options,
                            CompileStage::Hir,
                            Some(content_hash),
                            &intermediate_dir,
                        )?;
                        hir_session
                    };

                    if find_modules {
                        for module in hir_session.modules.iter() {
                            let module_name = module.name.unintern_or_default(&intermediate_dir);
                            tx_to_main.send(MessageToMain::AddModule {
                                path: input_module_path.join(module_name),
                                span: module.name_span,
                            })?;
                        }
                    }

                    if !hir_session.errors.is_empty() || stop_after <= CompileStage::Hir {
                        tx_to_main.send(MessageToMain::IrComplete {
                            module_path: Some(input_module_path),
                            compile_stage: CompileStage::Hir,
                            errors: hir_session.errors.clone(),
                            warnings: hir_session.warnings.clone(),
                        })?;

                        if !hir_session.errors.is_empty() {
                            return Err(Error::CompileError);
                        }

                        else {
                            continue;
                        }
                    }

                    // the inter-hir session must have been created at this point
                    let inter_hir_session_bytes = get_cached_ir(
                        &intermediate_dir,
                        CompileStage::InterHir,
                        None,
                    )?.ok_or(Error::IrCacheNotFound(CompileStage::InterHir))?;
                    let mut inter_hir_session = sodigy_inter_hir::Session::decode(&inter_hir_session_bytes)?;
                    inter_hir_session.intermediate_dir = intermediate_dir.clone();
                    let _ = inter_hir_session.resolve_module(&mut hir_session);
                    hir_session.errors.extend(inter_hir_session.errors.drain(..));
                    hir_session.warnings.extend(inter_hir_session.warnings.drain(..));

                    let mut mir_session = sodigy_mir::lower(hir_session, &inter_hir_session);
                    init_span_string_map_if_necessary(
                        &mut mir_session,
                        &emit_ir_options,
                        &intermediate_dir,
                        /* read_from_file: */ false,
                        /* write_to_file: */ false,
                    )?;
                    emit_irs_if_has_to(
                        &mir_session,
                        &emit_ir_options,
                        CompileStage::Mir,
                        Some(content_hash),
                        &intermediate_dir,
                    )?;

                    mir_session
                };

                if !mir_session.errors.is_empty() || stop_after <= CompileStage::Mir {
                    tx_to_main.send(MessageToMain::IrComplete {
                        module_path: Some(input_module_path),
                        compile_stage: CompileStage::Mir,
                        errors: mir_session.errors.clone(),
                        warnings: mir_session.warnings.clone(),
                    })?;

                    if !mir_session.errors.is_empty() {
                        return Err(Error::CompileError);
                    }

                    else {
                        continue;
                    }
                }

                // the inter-mir session must have been created at this point
                let inter_mir_session_bytes = get_cached_ir(
                    &intermediate_dir,
                    CompileStage::InterMir,
                    None,
                )?.ok_or(Error::IrCacheNotFound(CompileStage::InterMir))?;
                let mut inter_mir_session = sodigy_inter_mir::Session::decode(&inter_mir_session_bytes)?;
                mir_session.errors.extend(inter_mir_session.errors.drain(..));
                mir_session.warnings.extend(inter_mir_session.warnings.drain(..));
                mir_session.types = inter_mir_session.types.drain().collect();
                mir_session.generic_instances = inter_mir_session.generic_instances.drain().collect();

                let _ = sodigy_post_mir::lower_matches(&mut mir_session);

                init_span_string_map_if_necessary(
                    &mut mir_session,
                    &emit_ir_options,
                    &intermediate_dir,
                    /* read_from_file: */ true,
                    /* write_to_file: */ false,
                )?;
                emit_irs_if_has_to(
                    &mir_session,
                    &emit_ir_options,
                    CompileStage::PostMir,
                    Some(content_hash),
                    &intermediate_dir,
                )?;

                if !mir_session.errors.is_empty() || stop_after <= CompileStage::PostMir {
                    tx_to_main.send(MessageToMain::IrComplete {
                        module_path: Some(input_module_path),
                        compile_stage: CompileStage::PostMir,
                        errors: mir_session.errors.clone(),
                        warnings: mir_session.warnings.clone(),
                    })?;

                    if !mir_session.errors.is_empty() {
                        return Err(Error::CompileError);
                    }

                    else {
                        continue;
                    }
                }

                unreachable!()
            },
            Command::InterHir {
                modules,
                intermediate_dir,
                emit_ir_options,
            } => {
                let mut inter_hir_session = sodigy_inter_hir::Session::new(&intermediate_dir);

                for (path, span) in modules.iter() {
                    let file = File::from_module_path(
                        0,  // project_id
                        &path.to_string(),
                        &intermediate_dir,
                    )?.ok_or(Error::MiscError)?;
                    let content_hash = file.get_content_hash(&intermediate_dir)?;
                    let hir_session_bytes = get_cached_ir(
                        &intermediate_dir,
                        CompileStage::Hir,
                        Some(content_hash),
                    )?.ok_or(Error::IrCacheNotFound(CompileStage::Hir))?;
                    let mut hir_session = sodigy_hir::Session::decode(&hir_session_bytes)?;
                    hir_session.intermediate_dir = intermediate_dir.clone();
                    inter_hir_session.ingest(*span, hir_session);
                }

                if let Ok(()) = inter_hir_session.resolve_alias() {
                    let _ = inter_hir_session.resolve_poly();
                }

                emit_irs_if_has_to(
                    &inter_hir_session,
                    &emit_ir_options,
                    CompileStage::InterHir,
                    None,
                    &intermediate_dir,
                )?;
                tx_to_main.send(MessageToMain::IrComplete {
                    module_path: None,
                    compile_stage: CompileStage::InterHir,
                    errors: inter_hir_session.errors.clone(),
                    warnings: inter_hir_session.warnings.clone(),
                })?;
            },
            Command::InterMir {
                modules,
                intermediate_dir,
                emit_ir_options,
            } => {
                let mut merged_mir_session: Option<mir::Session> = None;

                for path in modules.keys() {
                    let file = File::from_module_path(
                        0,  // project_id
                        &path.to_string(),
                        &intermediate_dir,
                    )?.ok_or(Error::MiscError)?;
                    let content_hash = file.get_content_hash(&intermediate_dir)?;
                    let mir_session_bytes = get_cached_ir(
                        &intermediate_dir,
                        CompileStage::Mir,
                        Some(content_hash),
                    )?.ok_or(Error::IrCacheNotFound(CompileStage::Mir))?;
                    let mut mir_session = sodigy_mir::Session::decode(&mir_session_bytes)?;
                    mir_session.intermediate_dir = intermediate_dir.clone();

                    match &mut merged_mir_session {
                        Some(s) => {
                            s.merge(mir_session);
                        },
                        None => {
                            merged_mir_session = Some(mir_session);
                        },
                    }
                }

                let mir_session = merged_mir_session.unwrap();

                // `inter_mir_session` has type information of every items in the project.
                // It's relatively cheap to load/store, so post-mir and later stages will
                // use this session to get type information.
                //
                // `mir_session` has definition of every items, after poly-solving and
                // monomorphization. It's very heavy, and we're not gonna store this.
                let (inter_mir_session, mut mir_session) = sodigy_inter_mir::solve_type(mir_session);

                init_span_string_map_if_necessary(
                    &mut mir_session,
                    &emit_ir_options,
                    &intermediate_dir,
                    /* read_from_file: */ false,
                    /* write_to_file: */ true,
                )?;

                // InterMir may have modified MIRs, so we have to update all the cached MIRs.
                // NOTE: It drains the items in `mir_session`, so we cannot use the session anymore.
                // TODO: This is (potentially) one of the biggest bottleneck in the compiler.
                let items = mir_session.get_item_map();

                for path in modules.keys() {
                    let file = File::from_module_path(
                        0,  // project_id
                        &path.to_string(),
                        &intermediate_dir,
                    )?.ok_or(Error::MiscError)?;
                    let content_hash = file.get_content_hash(&intermediate_dir)?;
                    let mir_session_bytes = get_cached_ir(
                        &intermediate_dir,
                        CompileStage::Mir,
                        Some(content_hash),
                    )?.ok_or(Error::IrCacheNotFound(CompileStage::Mir))?;
                    let mut mir_session = sodigy_mir::Session::decode(&mir_session_bytes)?;
                    mir_session.intermediate_dir = intermediate_dir.clone();
                    mir_session.update_items(&items);
                    emit_irs_if_has_to(
                        &mir_session,
                        &[
                            EmitIrOption {
                                stage: CompileStage::Mir,
                                store: StoreIrAt::IntermediateDir,
                                human_readable: false,
                            },
                        ],
                        CompileStage::Mir,
                        Some(content_hash),
                        &intermediate_dir,
                    )?;
                }
                emit_irs_if_has_to(
                    &inter_mir_session,
                    &emit_ir_options,
                    CompileStage::InterMir,
                    None,
                    &intermediate_dir,
                )?;
                tx_to_main.send(MessageToMain::IrComplete {
                    module_path: None,
                    compile_stage: CompileStage::InterMir,
                    errors: inter_mir_session.errors,
                    warnings: inter_mir_session.warnings,
                })?;
            },
            Command::Bytecode {
                modules,
                intermediate_dir,
                optimize_level,
                backend,
                output_path,
                stop_after,
            } => {
                let mut merged_mir_session: Option<mir::Session> = None;

                for path in modules.keys() {
                    let file = File::from_module_path(
                        0,  // project_id
                        &path.to_string(),
                        &intermediate_dir,
                    )?.ok_or(Error::MiscError)?;
                    let content_hash = file.get_content_hash(&intermediate_dir)?;
                    let mir_session_bytes = get_cached_ir(
                        &intermediate_dir,
                        CompileStage::PostMir,
                        Some(content_hash),
                    )?.ok_or(Error::IrCacheNotFound(CompileStage::PostMir))?;
                    let mut mir_session = sodigy_mir::Session::decode(&mir_session_bytes)?;
                    mir_session.intermediate_dir = intermediate_dir.clone();

                    match &mut merged_mir_session {
                        Some(s) => {
                            s.merge(mir_session);
                        },
                        None => {
                            merged_mir_session = Some(mir_session);
                        },
                    }
                }

                let mir_session = merged_mir_session.unwrap();
                let optimized_mir_session = sodigy_optimize::optimize(mir_session, optimize_level);

                if !optimized_mir_session.errors.is_empty() || stop_after <= CompileStage::Optimize {
                    tx_to_main.send(MessageToMain::IrComplete {
                        module_path: None,
                        compile_stage: CompileStage::Optimize,
                        errors: optimized_mir_session.errors.clone(),
                        warnings: optimized_mir_session.warnings.clone(),
                    })?;

                    if !optimized_mir_session.errors.is_empty() {
                        return Err(Error::CompileError);
                    }

                    else {
                        continue;
                    }
                }

                let bytecode_session = sodigy_bytecode::lower(optimized_mir_session);

                if !bytecode_session.errors.is_empty() || stop_after <= CompileStage::Bytecode {
                    tx_to_main.send(MessageToMain::IrComplete {
                        module_path: None,
                        compile_stage: CompileStage::Bytecode,
                        errors: bytecode_session.errors.clone(),
                        warnings: bytecode_session.warnings.clone(),
                    })?;

                    if !bytecode_session.errors.is_empty() {
                        return Err(Error::CompileError);
                    }

                    else {
                        continue;
                    }
                }

                let (result, errors, warnings) = sodigy_code_gen::lower(bytecode_session, backend);

                match output_path {
                    StoreIrAt::File(f) => {
                        write_bytes(&f, &result, WriteMode::CreateOrTruncate)?;
                    },
                    StoreIrAt::IntermediateDir => {
                        emit_irs_if_has_to(
                            &result,
                            &[EmitIrOption {
                                stage: CompileStage::CodeGen,
                                store: StoreIrAt::IntermediateDir,
                                human_readable: false,
                            }],
                            CompileStage::CodeGen,
                            None,
                            &intermediate_dir,
                        )?;
                    },
                }

                tx_to_main.send(MessageToMain::IrComplete {
                    module_path: None,
                    compile_stage: CompileStage::CodeGen,
                    errors,
                    warnings,
                })?;
            },
        }
    }

    Ok(())
}

fn init_ir_dir(intermediate_dir: &str) -> Result<(), FileError> {
    let intern_str_map_dir = join(intermediate_dir, "str")?;
    let intern_num_map_dir = join(intermediate_dir, "num")?;
    let ir_dir = join(intermediate_dir, "irs")?;

    if !exists(&intern_str_map_dir) {
        create_dir_all(&intern_str_map_dir)?;
    }

    if !exists(&intern_num_map_dir) {
        create_dir_all(&intern_num_map_dir)?;
    }

    if !exists(&ir_dir) {
        create_dir_all(&ir_dir)?;

        for stage in COMPILE_STAGES {
            create_dir(&join(&ir_dir, &format!("{stage:?}").to_lowercase())?)?;
        }
    }

    File::clear_cache(0 /* project id */, intermediate_dir)?;
    Ok(())
}

fn emit_irs_if_has_to<T: Endec + DumpSession>(
    session: &T,
    emit_ir_options: &[EmitIrOption],
    finished_stage: CompileStage,
    content_hash: Option<u128>,
    intermediate_dir: &str,
) -> Result<(), Error> {
    let (mut binary, mut human_readable) = (false, false);
    let stores = emit_ir_options.iter().filter(
        |option| option.stage == finished_stage
    ).map(
        |option| {
            if option.human_readable {
                human_readable = true;
            } else {
                binary = true;
            }

            (option.store.clone(), option.human_readable)
        }
    ).collect::<Vec<_>>();
    let binary = if binary {
        Some(session.encode())
    } else {
        None
    };
    let human_readable = if human_readable {
        Some(session.dump_session())
    } else {
        None
    };

    for (store, human_readable_) in stores.iter() {
        let content = if *human_readable_ {
            human_readable.as_ref().unwrap()
        } else {
            binary.as_ref().unwrap()
        };
        let ext = if *human_readable_ { ".rs" } else { "" };

        match store {
            StoreIrAt::File(s) => {
                write_bytes(&s, content, WriteMode::Atomic)?;
            },
            StoreIrAt::IntermediateDir => {
                let path = join4(
                    intermediate_dir,
                    "irs",
                    &format!("{finished_stage:?}").to_lowercase(),
                    &format!(
                        "{}{ext}",
                        if let Some(content_hash) = content_hash {
                            format!("{content_hash:x}")
                        } else {
                            String::from("total")
                        },
                    ),
                )?;
                let parent = parent(&path)?;

                if !exists(&parent) {
                    create_dir(&parent)?;
                }

                write_bytes(
                    &path,
                    content,
                    WriteMode::Atomic,
                )?;
            },
        }
    }

    Ok(())
}

fn init_span_string_map_if_necessary(
    session: &mut mir::Session,
    emit_ir_options: &[EmitIrOption],
    intermediate_dir: &str,
    read_from_file: bool,
    write_to_file: bool,
) -> Result<(), Error> {
    for option in emit_ir_options.iter() {
        match option {
            EmitIrOption {
                stage: CompileStage::Mir | CompileStage::InterMir | CompileStage::PostMir,
                human_readable: true,
                ..
            } => {
                let path = join3(
                    intermediate_dir,
                    "irs",
                    "span_string_map",
                )?;

                if read_from_file {
                    let bytes = read_bytes(&path)?;
                    session.span_string_map = Some(HashMap::<_, _>::decode(&bytes)?);
                }

                else {
                    session.init_span_string_map();
                }

                if write_to_file {
                    let Some(span_string_map) = &session.span_string_map else { unreachable!() };
                    let bytes = span_string_map.encode();
                    write_bytes(&path, &bytes, WriteMode::CreateOrTruncate)?;
                }

                break;
            },
            _ => {},
        }
    }

    Ok(())
}

fn get_cached_ir(
    intermediate_dir: &str,
    stage: CompileStage,
    content_hash: Option<u128>,
) -> Result<Option<Vec<u8>>, FileError> {
    let path = join4(
        intermediate_dir,
        "irs",
        &format!("{stage:?}").to_lowercase(),
        // There's no `ext` because it's always `!human_readable`
        &if let Some(content_hash) = content_hash {
            format!("{content_hash:x}")
        } else {
            String::from("total")
        },
    )?;

    if exists(&path) {
        Ok(Some(read_bytes(&path)?))
    }

    else {
        Ok(None)
    }
}

// I want purely functional `push` method, but rust doesn't have one. So I created one!
trait CloneAndPush<T> {
    fn clone_and_push(&self, element: T) -> Vec<T>;
}

impl<T: Clone> CloneAndPush<T> for Vec<T> {
    fn clone_and_push(&self, element: T) -> Vec<T> {
        let mut r = self.to_vec();
        r.push(element);
        r
    }
}
