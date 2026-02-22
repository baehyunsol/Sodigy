use sodigy::{
    Command,
    EmitIrOption,
    Error,
    ModuleCompileState,
    StoreIrAt,
    get_cached_ir,
};
use sodigy_code_gen::Backend;
use sodigy_driver::{
    CliCommand,
    ColorWhen,
    CompileStage,
    COMPILE_STAGES,
    Profile,
    parse_args,
};
use sodigy_endec::Endec;
use sodigy_error::{
    CustomErrorLevel,
    DumpErrorOption,
    Error as SodigyError,
    ErrorLevel,
    Warning as SodigyWarning,
};
use sodigy_file::{File, ModulePath};
use sodigy_fs_api::{
    FileError,
    FileErrorKind,
    WriteMode,
    basename,
    create_dir,
    create_dir_all,
    exists,
    join,
    read_bytes,
    read_dir,
    remove_dir,
    set_current_dir,
    write_string,
};
use sodigy_optimize::OptimizeLevel;
use sodigy_span::{Color, Span};
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Instant;

mod timings;
mod worker;

use timings::{TimingsEntry, dump_timings};
use worker::{Channel, MessageToMain, MessageToWorker, Worker, WorkerId, init_workers_and_channels};

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
                            format!("\n\n{}", sodigy_cli::underline_span(span))
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

    match &parse_args(&args)? {
        CliCommand::New { project_name } => {
            init_project(project_name)?;
            Ok(())
        },
        cli_command @ (
            CliCommand::Build { optimize_level, import_std, custom_error_levels, emit_irs, graceful_shutdown, jobs, color, .. } |
            CliCommand::Run { optimize_level, import_std, custom_error_levels, emit_irs, graceful_shutdown, jobs, color } |
            CliCommand::Test { optimize_level, import_std, custom_error_levels, emit_irs, graceful_shutdown, jobs, color }
        ) => {
            let started_at = Instant::now();
            let mut errors = vec![];
            let mut warnings = vec![];
            let mut worker_logs = HashMap::new();
            let channels = init_workers_and_channels(*jobs);
            let (output_path, backend) = match cli_command {
                CliCommand::Run { .. } => (StoreIrAt::IntermediateDir, Backend::Bytecode),
                CliCommand::Test { .. } => (StoreIrAt::IntermediateDir, Backend::Bytecode),
                CliCommand::Build { output_path, backend, .. } => (StoreIrAt::File(output_path.to_string()), *backend),
                _ => todo!(),
            };

            let result = compile(
                output_path,
                backend,
                ir_dir.clone(),
                *optimize_level,
                *import_std,
                &custom_error_levels,
                *emit_irs,
                *graceful_shutdown,

                // TODO: make it configurable
                true,  // incremental_compilation

                &channels,
                &mut errors,
                &mut warnings,
                &mut worker_logs,
            );

            let elapsed_ms = Instant::now().duration_since(started_at).as_millis();
            let dump_error_option = match color {
                // TODO: `ColorWhen::Auto` is WIP
                ColorWhen::Auto | ColorWhen::Always => DumpErrorOption::default(),
                ColorWhen::Never => DumpErrorOption {
                    error_color: Color::None,
                    warning_color: Color::None,
                    auxiliary_color: Color::None,
                    info_color: Color::None,
                    ..DumpErrorOption::default()
                },
            };

            apply_custom_error_levels(
                &custom_error_levels,
                &mut errors,
                &mut warnings,
            );
            sodigy_error::dump_errors(
                errors,
                warnings,
                &ir_dir,
                dump_error_option,
                Some(elapsed_ms as u64),
            );
            let mut all_worker_ids = Vec::with_capacity(channels.len());

            for channel in channels.iter() {
                let _ = channel.send(MessageToWorker::Kill);
                all_worker_ids.push(channel.worker_id);
            }

            for channel in channels.into_iter() {
                let worker_id = channel.worker_id;

                // Erroneous workers are already dead and their logs are already collected.
                // The other workers' logs are collected here.
                if let Some(worker_log) = channel.join() {
                    worker_logs.insert(worker_id, worker_log);
                }
            }

            // TODO: make it configurable
            dump_timings(all_worker_ids, &worker_logs, &ir_dir)?;
            result?;

            match cli_command {
                CliCommand::Run { .. } => interpret(StoreIrAt::IntermediateDir, Profile::Script, &ir_dir),
                CliCommand::Test { .. } => interpret(StoreIrAt::IntermediateDir, Profile::Test, &ir_dir),
                _ => Ok(()),
            }
        },
        CliCommand::Interpret { bytecodes_path } => interpret(
            StoreIrAt::File(bytecodes_path.to_string()),

            // TODO: make it configurable
            Profile::Test,

            // intermediate_dir not needed
            "",
        ),
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
    custom_error_levels: &HashMap<u16, CustomErrorLevel>,
    emit_irs: bool,
    graceful_shutdown: u32,  // in milliseconds
    incremental_compilation: bool,
    workers: &[Channel],
    errors: &mut Vec<SodigyError>,
    warnings: &mut Vec<SodigyWarning>,
    worker_logs: &mut HashMap<WorkerId, Vec<TimingsEntry>>,
) -> Result<(), Error> {
    goto_root_dir()?;
    let mut shutdown_countdown: Option<Instant> = None;
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
            CompileStage::MirOptimize,
            CompileStage::Bytecode,
            CompileStage::BytecodeOptimize,
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
    init_ir_dir(&ir_dir, incremental_compilation)?;

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
        // TODO: It has to immediately return if no worker's working.
        //       Naively checking `modules.all(|m| m.running)` isn't enough because
        //       an errorneous worker won't change its status and there can be
        //       multiple erroneous workers!
        if let Some(started_at) = &shutdown_countdown {
            if Instant::now().duration_since(started_at.clone()).as_millis() >= graceful_shutdown as u128 {
                return Err(Error::CompileError);
            }
        }

        let mut every_hir_complete = true;
        let mut every_mir_complete = true;
        let mut every_bytecode_complete = true;

        for module in modules.values_mut() {
            if let (CompileStage::Load, false) = (module.compile_stage, module.running) {
                workers[round_robin % workers.len()].send(MessageToWorker::Run(vec![
                    Command::PerFileIr {
                        input_file_path: module.file_path.clone(),
                        input_module_path: module.module_path.clone(),
                        optimize_level,
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

            if (module.compile_stage, module.running) != (CompileStage::BytecodeOptimize, false) {
                every_bytecode_complete = false;
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

        if every_bytecode_complete {
            workers[round_robin % workers.len()].send(MessageToWorker::Run(vec![
                Command::CodeGen {
                    modules: modules.values().map(
                        |module| (module.module_path.clone(), module.span)
                    ).collect(),
                    intermediate_dir: ir_dir.clone(),
                    backend,
                    output_path: output_path.clone(),
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

                        if !errors.is_empty() || has_forbidden_warning(warnings, custom_error_levels) {
                            // There's only 1 worker, so graceful shutdown doesn't make sense!
                            if compile_stage == CompileStage::InterHir || compile_stage == CompileStage::InterMir {
                                return Err(Error::CompileError);
                            }

                            if shutdown_countdown.is_none() {
                                shutdown_countdown = Some(Instant::now());
                            }

                            continue;
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
                                            optimize_level,
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
                                            optimize_level,
                                            intermediate_dir: ir_dir.clone(),
                                            find_modules: false,
                                            emit_ir_options: emit_irs.clone_and_push(
                                                EmitIrOption {
                                                    stage: CompileStage::BytecodeOptimize,
                                                    store: StoreIrAt::IntermediateDir,
                                                    human_readable: false,
                                                },
                                            ),
                                            stop_after: CompileStage::BytecodeOptimize,
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
                    MessageToMain::TimingsLog { worker_id, entries } => {
                        worker_logs.insert(worker_id, entries);
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

fn init_ir_dir(
    intermediate_dir: &str,
    incremental_compilation: bool,
) -> Result<(), FileError> {
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
            let dir_path = &join(&ir_dir, &format!("{stage:?}").to_lowercase())?;

            // We have to reuse irs from previous compilations -> incremental compilation.
            // But we should not use Mirs from previous ones, because mirs are generated
            // after inter-hir.
            // TODO: We have to reuse everything if nothing's changed.
            if stage > CompileStage::Hir || !incremental_compilation {
                if exists(&dir_path) {
                    remove_dir(&dir_path)?;
                }
            }

            create_dir(dir_path)?;
        }
    }

    // TODO: What's the point of incremental compilation if we clear cache every time?
    File::clear_cache(0 /* project id */, intermediate_dir)?;
    Ok(())
}

fn apply_custom_error_levels(
    custom_error_levels: &HashMap<u16, CustomErrorLevel>,
    errors: &mut Vec<SodigyError>,
    warnings: &mut Vec<SodigyWarning>,
) {
    let mut new_warnings = Vec::with_capacity(warnings.len());

    for warning in warnings.drain(..) {
        match ErrorLevel::from_error_kind(&warning.kind) {
            ErrorLevel::Error => unreachable!(),
            l @ (ErrorLevel::Warning | ErrorLevel::Lint) => match custom_error_levels.get(&warning.kind.index()) {
                Some(CustomErrorLevel::Forbid) => {
                    errors.push(warning);
                },
                Some(CustomErrorLevel::Warn) => {
                    new_warnings.push(warning);
                },
                Some(CustomErrorLevel::Allow) => {},
                None => match l {
                    ErrorLevel::Error => unreachable!(),
                    ErrorLevel::Warning => {
                        new_warnings.push(warning);
                    },
                    ErrorLevel::Lint => {},
                },
            },
        }
    }

    *warnings = new_warnings;
}

fn has_forbidden_warning(
    warnings: &[SodigyWarning],
    custom_error_levels: &HashMap<u16, CustomErrorLevel>,
) -> bool {
    for warning in warnings.iter() {
        match custom_error_levels.get(&warning.kind.index()) {
            Some(CustomErrorLevel::Forbid) => {
                return true;
            },
            _ => {},
        }
    }

    false
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
