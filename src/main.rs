use sodigy::{
    Backend,
    CliCommand,
    Command,
    CompileStage,
    COMPILE_STAGES,
    EmitIrOption,
    Error,
    Profile,
    QuickError,
    StoreIrAt,
    parse_args,
};
use sodigy_endec::{DumpIr, Endec};
use sodigy_error::Error as SodigyError;
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
    join4,
    parent,
    read_bytes,
    read_dir,
    set_current_dir,
    write_bytes,
    write_string,
};
use sodigy_hir as hir;
use sodigy_mir::Session as MirSession;
use sodigy_session::Session;
use sodigy_span::Span;
use sodigy_string::unintern_string;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

mod worker;

use worker::{MessageToMain, MessageToWorker};

fn main() -> Result<(), Error> {
    let args = std::env::args().collect::<Vec<_>>();

    match parse_args(&args) {
        Ok(command) => match command {
            CliCommand::New { project_name } => {
                init_project(&project_name).map_err(|e| Error::FileError(e))?;
                Ok(())
            },
            CliCommand::Test {
                optimization,
                import_std,
                jobs,
            } => {
                goto_root_dir()?;
                let workers = worker::init_workers(jobs);
                let mut run_id = 0;
                let mut unfinished_runs = HashSet::new();

                // HashMap<path of the module, def_span of the module>
                let mut generated_hirs: HashMap<ModulePath, Span> = HashMap::new();

                let input_module_path = ModulePath::lib();
                let input_file_path = input_module_path.get_file_path().map_err(
                    |e| SodigyError {
                        kind: e.into(),
                        spans: Span::Lib.simple_error(),
                        note: None,
                    }
                ).continue_or_dump_error("target")?;
                generated_hirs.insert(input_module_path.clone(), Span::Lib);

                workers[run_id % workers.len()].send(MessageToWorker::Run {
                    commands: vec![
                        Command::InitIrDir {
                            intermediate_dir: String::from("target"),
                        },
                        Command::PerFileIr {
                            input_file_path,
                            input_module_path,
                            intermediate_dir: String::from("target"),
                            find_modules: true,
                            emit_ir_options: vec![
                                // for debugging
                                EmitIrOption {
                                    stage: CompileStage::Lex,
                                    store: StoreIrAt::IntermediateDir,
                                    human_readable: true,
                                },
                                EmitIrOption {
                                    stage: CompileStage::Parse,
                                    store: StoreIrAt::IntermediateDir,
                                    human_readable: true,
                                },
                                EmitIrOption {
                                    stage: CompileStage::Hir,
                                    store: StoreIrAt::IntermediateDir,
                                    human_readable: true,
                                },

                                // cache hir for incremental compilation
                                EmitIrOption {
                                    stage: CompileStage::Hir,
                                    store: StoreIrAt::IntermediateDir,
                                    human_readable: false,
                                },
                            ],
                            stop_after: CompileStage::Hir,
                        },
                    ],
                    id: run_id,
                })?;
                unfinished_runs.insert(run_id);
                run_id += 1;

                // compile std
                if import_std {
                    let (input_module_path, input_file_path) = sodigy_file::std_root();
                    generated_hirs.insert(input_module_path.clone(), Span::Std);
                    workers[run_id % workers.len()].send(MessageToWorker::Run {
                        commands: vec![
                            Command::PerFileIr {
                                input_file_path,
                                input_module_path,
                                intermediate_dir: String::from("target"),
                                find_modules: true,
                                emit_ir_options: vec![
                                    // cache hir for incremental compilation
                                    EmitIrOption {
                                        stage: CompileStage::Hir,
                                        store: StoreIrAt::IntermediateDir,
                                        human_readable: false,
                                    },
                                ],
                                stop_after: CompileStage::Hir,
                            },
                        ],
                        id: run_id,
                    })?;
                    unfinished_runs.insert(run_id);
                    run_id += 1;
                }

                // loop 1: generate hir of all files
                loop {
                    for worker in workers.iter() {
                        match worker.try_recv() {
                            Ok(msg) => match msg {
                                MessageToMain::FoundModuleDef {
                                    path,
                                    span,
                                } => {
                                    if !generated_hirs.contains_key(&path) {
                                        generated_hirs.insert(path.clone(), span);
                                        let file_path = path.get_file_path().map_err(
                                            |e| SodigyError {
                                                kind: e.into(),
                                                spans: span.simple_error(),
                                                note: None,
                                            }
                                        ).continue_or_dump_error("target")?;
                                        workers[run_id % workers.len()].send(MessageToWorker::Run {
                                            commands: vec![Command::PerFileIr {
                                                input_file_path: file_path,
                                                input_module_path: path,
                                                intermediate_dir: String::from("target"),
                                                find_modules: true,
                                                emit_ir_options: vec![
                                                    EmitIrOption {
                                                        stage: CompileStage::Hir,
                                                        store: StoreIrAt::IntermediateDir,
                                                        human_readable: false,
                                                    },
                                                ],
                                                stop_after: CompileStage::Hir,
                                            }],
                                            id: run_id,
                                        })?;
                                        unfinished_runs.insert(run_id);
                                        run_id += 1;
                                    }
                                },
                                MessageToMain::RunComplete { id } => {
                                    unfinished_runs.remove(&id);
                                },
                                MessageToMain::Error { id, e } => {
                                    unfinished_runs.remove(&id);

                                    // Kinda graceful shutdown, so that workers can dump their error messages
                                    if !unfinished_runs.is_empty() {
                                        thread::sleep(Duration::from_millis(500));
                                    }

                                    return Err(e);
                                },
                            },
                            Err(mpsc::TryRecvError::Empty) => {},
                            Err(mpsc::TryRecvError::Disconnected) => {
                                return Err(Error::MpscError);
                            },
                        }
                    }

                    if unfinished_runs.is_empty() {
                        break;
                    }

                    thread::sleep(Duration::from_millis(100));
                }

                workers[run_id % workers.len()].send(MessageToWorker::Run {
                    commands: vec![Command::InterHir {
                        modules: generated_hirs.clone(),
                        intermediate_dir: String::from("target"),
                    }],
                    id: run_id,
                })?;
                unfinished_runs.insert(run_id);
                run_id += 1;

                // loop 2: generate inter-hir map
                loop {
                    for worker in workers.iter() {
                        match worker.try_recv() {
                            Ok(msg) => match msg {
                                MessageToMain::FoundModuleDef { .. } => unreachable!(),
                                MessageToMain::RunComplete { id } => {
                                    unfinished_runs.remove(&id);
                                },
                                MessageToMain::Error { id, e } => {
                                    unfinished_runs.remove(&id);

                                    // Kinda graceful shutdown, so that workers can dump their error messages
                                    if !unfinished_runs.is_empty() {
                                        thread::sleep(Duration::from_millis(500));
                                    }

                                    return Err(e);
                                },
                            },
                            Err(mpsc::TryRecvError::Empty) => {},
                            Err(mpsc::TryRecvError::Disconnected) => {
                                return Err(Error::MpscError);
                            },
                        }
                    }

                    if unfinished_runs.is_empty() {
                        break;
                    }

                    thread::sleep(Duration::from_millis(100));
                }

                for (path, span) in generated_hirs.iter() {
                    let file_path = path.get_file_path().map_err(
                        |e| SodigyError {
                            kind: e.into(),
                            spans: span.simple_error(),
                            note: None,
                        }
                    ).continue_or_dump_error("target")?;
                    workers[run_id % workers.len()].send(MessageToWorker::Run {
                        commands: vec![Command::PerFileIr {
                            input_file_path: file_path,
                            input_module_path: path.clone(),
                            intermediate_dir: String::from("target"),
                            find_modules: false,
                            emit_ir_options: vec![
                                EmitIrOption {
                                    stage: CompileStage::Mir,
                                    store: StoreIrAt::IntermediateDir,
                                    human_readable: false,
                                },

                                // for debugging
                                EmitIrOption {
                                    stage: CompileStage::Mir,
                                    store: StoreIrAt::IntermediateDir,
                                    human_readable: true,
                                },
                            ],
                            stop_after: CompileStage::Mir,
                        }],
                        id: run_id,
                    })?;
                    unfinished_runs.insert(run_id);
                    run_id += 1;
                }

                // loop 3: generate mir of all files
                loop {
                    for worker in workers.iter() {
                        match worker.try_recv() {
                            Ok(msg) => match msg {
                                MessageToMain::FoundModuleDef { .. } => unreachable!(),
                                MessageToMain::RunComplete { id } => {
                                    unfinished_runs.remove(&id);
                                },
                                MessageToMain::Error { id, e } => {
                                    unfinished_runs.remove(&id);

                                    // Kinda graceful shutdown, so that workers can dump their error messages
                                    if !unfinished_runs.is_empty() {
                                        thread::sleep(Duration::from_millis(500));
                                    }

                                    return Err(e);
                                },
                            },
                            Err(mpsc::TryRecvError::Empty) => {},
                            Err(mpsc::TryRecvError::Disconnected) => {
                                return Err(Error::MpscError);
                            },
                        }
                    }

                    if unfinished_runs.is_empty() {
                        break;
                    }

                    thread::sleep(Duration::from_millis(100));
                }

                workers[0].send(MessageToWorker::Run {
                    commands: vec![
                        Command::InterMir {
                            modules: generated_hirs.clone(),
                            intermediate_dir: String::from("target"),
                            stop_after: CompileStage::CodeGen,
                            emit_ir_options: vec![
                                EmitIrOption {
                                    stage: CompileStage::CodeGen,
                                    store: StoreIrAt::Memory,
                                    human_readable: false,
                                },
                                // for debugging
                                EmitIrOption {
                                    stage: CompileStage::Bytecode,
                                    store: StoreIrAt::IntermediateDir,
                                    human_readable: true,
                                },
                            ],
                            dump_type_info: false,  // enable this to debug the type checker!
                            output_path: None,
                            backend: Backend::Bytecode,
                            profile: Profile::Test,
                            optimization,
                        },
                        Command::Interpret {
                            bytecodes_path: StoreIrAt::Memory,
                            profile: Profile::Test,
                        },
                    ],
                    id: run_id,
                })?;

                match workers[0].recv() {
                    Ok(msg) => match msg {
                        MessageToMain::FoundModuleDef { .. } => unreachable!(),
                        MessageToMain::RunComplete { .. } => Ok(()),
                        MessageToMain::Error { e, .. } => Err(e),
                    },
                    Err(_) => Err(Error::MpscError),
                }
            },
            _ => panic!("TODO: {command:?}"),
        },
        Err(e) => {
            let message = e.kind.render();

            eprintln!("cli error: {message}{}",
                if let Some(span) = e.span {
                    format!("\n\n{}", ragit_cli::underline_span(&span))
                } else {
                    String::new()
                },
            );
            Err(Error::CliError)
        },
    }
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
    write_string(
        &lib,
        "fn add(x: Int, y: Int) -> Int = x + y;",
        WriteMode::CreateOrTruncate,
    )?;

    let main = join(&src, "main.sdgsh")?;
    write_string(
        &main,
        "add 1 1 | print;",
        WriteMode::CreateOrTruncate,
    )?;

    let config = join(&name, "sodigy.toml")?;
    write_string(
        &config,
        "TODO",
        WriteMode::CreateOrTruncate,
    )?;
    Ok(())
}

pub fn run(commands: Vec<Command>, tx_to_main: mpsc::Sender<MessageToMain>) -> Result<(), Error> {
    let mut memory = None;

    for command in commands.into_iter() {
        match command {
            Command::InitIrDir {
                intermediate_dir,
            } => init_ir_dir(&intermediate_dir)?,
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
                let mut cached_hir_session = None;

                if let Some(cached_data) = get_cached_ir(
                    &intermediate_dir,
                    CompileStage::Hir,
                    Some(content_hash),
                )? {
                    // TODO: It doesn't have to exit at decode_error, it can just generate hir from scratch.
                    //       But then, it'd be impossible to catch this error. I'm still debugging the compiler
                    //       so I'll just let it crash.
                    let mut s = hir::Session::decode(&cached_data)?;
                    s.intermediate_dir = intermediate_dir.clone();
                    cached_hir_session = Some(s);
                }

                let mut hir_session = if let Some(mut hir_session) = cached_hir_session {
                    hir_session.intermediate_dir = intermediate_dir.clone();
                    hir_session
                } else {
                    // TODO: throw an ICE instead of unwrap
                    let bytes = file.read_bytes(&intermediate_dir)?.unwrap();

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
                        &mut memory,
                    )?;
                    lex_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                    if let CompileStage::Lex = stop_after {
                        continue;
                    }

                    let parse_session = sodigy_parse::parse(lex_session);
                    emit_irs_if_has_to(
                        &parse_session,
                        &emit_ir_options,
                        CompileStage::Parse,
                        Some(content_hash),
                        &intermediate_dir,
                        &mut memory,
                    )?;
                    parse_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                    if let CompileStage::Parse = stop_after {
                        continue;
                    }

                    let hir_session = sodigy_hir::lower(parse_session);
                    emit_irs_if_has_to(
                        &hir_session,
                        &emit_ir_options,
                        CompileStage::Hir,
                        Some(content_hash),
                        &intermediate_dir,
                        &mut memory,
                    )?;
                    hir_session
                };

                hir_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                if find_modules {
                    for module in hir_session.modules.iter() {
                        let module_name = unintern_string(module.name, &intermediate_dir)?.unwrap();
                        let module_name = String::from_utf8_lossy(&module_name).to_string();
                        tx_to_main.send(MessageToMain::FoundModuleDef {
                            path: input_module_path.join(module_name),
                            span: module.name_span,
                        })?;
                    }
                }

                if let CompileStage::Hir = stop_after {
                    continue;
                }

                // the inter-hir session must have been created at this point
                let inter_hir_session = get_cached_ir(
                    &intermediate_dir,
                    CompileStage::InterHir,
                    None,
                )?.unwrap();  // TODO: throw an ICE instead of unwrapping
                let mut inter_hir_session = sodigy_inter_hir::Session::decode(&inter_hir_session)?;
                inter_hir_session.intermediate_dir = intermediate_dir.clone();
                let _ = inter_hir_session.resolve_module(&mut hir_session);
                hir_session.errors.extend(inter_hir_session.errors.drain(..));
                hir_session.warnings.extend(inter_hir_session.warnings.drain(..));
                hir_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                // TODO: Now that inter_hir_session and hir_session are updated, we have to cache them again.
                //       Be careful not to overwrite the per-file hir sessions. (do we have to create another CompileStage for this?)

                if let CompileStage::InterHir = stop_after {
                    continue;
                }

                let mir_session = sodigy_mir::lower(hir_session, &inter_hir_session);
                emit_irs_if_has_to(
                    &mir_session,
                    &emit_ir_options,
                    CompileStage::Mir,
                    Some(content_hash),
                    &intermediate_dir,
                    &mut memory,
                )?;
                mir_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                if let CompileStage::Mir = stop_after {
                    continue;
                }
            },
            Command::InterHir {
                modules,
                intermediate_dir,
            } => {
                let mut inter_hir_session = sodigy_inter_hir::Session::new(&intermediate_dir);

                for (path, span) in modules.iter() {
                    let file = File::from_module_path(
                        0,  // project_id
                        &path.to_string(),
                        &intermediate_dir,
                    )?.unwrap();  // TODO: throw an ICE instead of unwrapping it
                    let content_hash = file.get_content_hash(&intermediate_dir)?;
                    let hir_session_bytes = get_cached_ir(
                        &intermediate_dir,
                        CompileStage::Hir,
                        Some(content_hash),
                    )?;

                    let mut hir_session = match hir_session_bytes.map(|bytes| sodigy_hir::Session::decode(&bytes)) {
                        Some(Ok(session)) => session,

                        // TODO: It's kinda ICE, but there's no interface for ICE yet
                        _ => todo!(),
                    };

                    hir_session.intermediate_dir = intermediate_dir.clone();
                    inter_hir_session.ingest(*span, hir_session);
                }

                if let Ok(()) = inter_hir_session.resolve_alias() {
                    let _ = inter_hir_session.resolve_poly();
                }

                emit_irs_if_has_to(
                    &inter_hir_session,
                    &[
                        EmitIrOption {
                            stage: CompileStage::InterHir,
                            store: StoreIrAt::IntermediateDir,
                            human_readable: false,
                        },

                        // debug
                        EmitIrOption {
                            stage: CompileStage::InterHir,
                            store: StoreIrAt::IntermediateDir,
                            human_readable: true,
                        },
                    ],
                    CompileStage::InterHir,
                    None,
                    &intermediate_dir,
                    &mut memory,
                )?;
                inter_hir_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;
            },
            Command::InterMir {
                modules,
                intermediate_dir,
                stop_after,
                emit_ir_options,
                dump_type_info,
                output_path,
                backend,
                profile,
                optimization,
            } => {
                let mut merged_mir_session: Option<MirSession> = None;

                for path in modules.keys() {
                    let file = File::from_module_path(
                        0,  // project_id
                        &path.to_string(),
                        &intermediate_dir,
                    )?.unwrap();  // TODO: throw an ICE instead of unwrapping it
                    let content_hash = file.get_content_hash(&intermediate_dir)?;
                    let mir_session_bytes = get_cached_ir(
                        &intermediate_dir,
                        CompileStage::Mir,
                        Some(content_hash),
                    )?;

                    let mut mir_session = match mir_session_bytes.map(|bytes| sodigy_mir::Session::decode(&bytes)) {
                        Some(Ok(session)) => session,

                        // TODO: It's kinda ICE, but there's no interface for ICE yet
                        _ => todo!(),
                    };
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

                // TODO: dump type_solver
                let (mut mir_session, type_solver) = sodigy_mir_type::solve(mir_session, dump_type_info);

                if dump_type_info {
                    sodigy_mir_type::dump(&mut mir_session, &type_solver);
                }

                mir_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                if let CompileStage::TypeCheck = stop_after {
                    continue;
                }

                let mut lir_session = sodigy_lir::lower(mir_session);
                emit_irs_if_has_to(
                    &lir_session,
                    &emit_ir_options,
                    CompileStage::Bytecode,
                    None,
                    &intermediate_dir,
                    &mut memory,
                )?;
                lir_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                if let CompileStage::Bytecode = stop_after {
                    continue;
                }

                let executable = lir_session.into_executable();

                let result = match backend {
                    // Backend::Python => sodigy_backend::python_code_gen(
                    //     &executable,
                    //     &sodigy_backend::CodeGenConfig {
                    //         intermediate_dir: intermediate_dir.clone(),
                    //         label_help_comment: true,
                    //         mode: profile.into(),
                    //     },
                    // )?,
                    Backend::Bytecode => executable.encode(),
                    _ => todo!(),
                };

                emit_irs_if_has_to(
                    &result,
                    &emit_ir_options,
                    CompileStage::CodeGen,
                    None,
                    &intermediate_dir,
                    &mut memory,
                )?;

                lir_session.dump_warnings();

                if let Some(output_path) = output_path {
                    write_bytes(
                        &output_path,
                        &result,
                        WriteMode::CreateOrTruncate,
                    )?;
                }
            },
            Command::Interpret {
                bytecodes_path,
                profile,
            } => {
                let bytecodes_bytes = match bytecodes_path {
                    StoreIrAt::File(path) => read_bytes(&path)?,
                    // TODO: raise a FileNotFound error instead of unwrapping it
                    StoreIrAt::Memory => memory.clone().unwrap(),
                    StoreIrAt::IntermediateDir => todo!(),
                };
                let bytecodes_bytes = Vec::<u8>::decode(&bytecodes_bytes).unwrap();
                let executable = sodigy_lir::Executable::decode(&bytecodes_bytes)?;

                for (name, label) in executable.asserts.iter() {
                    sodigy_interpreter::interpret(&executable, *label).unwrap();
                }

                // match profile {
                //     Profile::Test => {
                //         let mut failed = false;

                //         // TODO: it has to capture stderr and stdout
                //         for (id, name) in bytecodes.asserts.iter() {
                //             if let Err(_) = sodigy_backend::interpret(
                //                 &bytecodes.bytecodes,
                //                 *id,
                //             ) {
                //                 println!("{name}: \x1b[31mFail\x1b[0m");
                //                 failed = true;
                //             }

                //             else {
                //                 println!("{name}: \x1b[32mPass\x1b[0m");
                //             }
                //         }

                //         if failed {
                //             return Err(Error::TestError);
                //         }
                //     },
                //     // It's TODO until we design and implement the impure part
                //     _ => {
                //         todo!()
                //     },
                // }
            },
            Command::Help(doc) => match doc {
                _ => todo!(),
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

fn emit_irs_if_has_to<T: Endec + DumpIr>(
    session: &T,
    emit_ir_options: &[EmitIrOption],
    finished_stage: CompileStage,
    content_hash: Option<u128>,
    intermediate_dir: &str,
    memory: &mut Option<Vec<u8>>,
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
        Some(session.dump_ir())
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
            StoreIrAt::Memory => {
                *memory = Some(content.to_vec());
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
