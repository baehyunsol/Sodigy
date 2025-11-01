use sodigy::{
    Backend,
    CliCommand,
    Command,
    CompileStage,
    EmitIrOption,
    Error,
    Optimization,
    Profile,
    StoreIrAt,
    parse_args,
};
use sodigy_endec::{DumpIr, Endec};
use sodigy_error::{Error as SodigyError, ErrorKind as SodigyErrorKind};
use sodigy_file::File;
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
    read_bytes,
    read_dir,
    set_current_dir,
    write_bytes,
    write_string,
};
use sodigy_hir as hir;
use sodigy_lir::Executable;
use sodigy_name_analysis::{IdentWithOrigin, NameOrigin};
use sodigy_session::{DummySession, Session};
use sodigy_span::Span;
use sodigy_string::intern_string;
use std::collections::HashSet;
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
                jobs,
            } => {
                goto_root_dir()?;
                let workers = worker::init_workers(jobs);
                let mut run_id = 0;
                let mut unfinished_runs = HashSet::new();
                let mut generated_hirs = HashSet::new();
                generated_hirs.insert(String::from("lib"));

                workers[run_id % workers.len()].send(MessageToWorker::Run {
                    commands: vec![
                        Command::InitIrDir {
                            intermediate_dir: String::from("target"),
                        },
                        Command::Compile {
                            // TODO: how about `src/lib/mod.sdg`? Does rust allow this?
                            // TODO: raise an error if `src/lib.sdg` does not exist
                            //       instead of raising FileError, it has to raise a more helpful one
                            input_file_path: String::from("src/lib.sdg"),
                            input_module_path: String::from("lib"),
                            intermediate_dir: String::from("target"),
                            emit_ir_options: vec![
                                // for debugging
                                EmitIrOption {
                                    stage: CompileStage::Lex,
                                    store: StoreIrAt::File(String::from("tokens.rs")),
                                    human_readable: true,
                                },
                                EmitIrOption {
                                    stage: CompileStage::Parse,
                                    store: StoreIrAt::File(String::from("ast.rs")),
                                    human_readable: true,
                                },
                                EmitIrOption {
                                    stage: CompileStage::Hir,
                                    store: StoreIrAt::File(String::from("hir.rs")),
                                    human_readable: true,
                                },

                                // cache hir for incremental compilation
                                EmitIrOption {
                                    stage: CompileStage::Hir,
                                    store: StoreIrAt::IntermediateDir,
                                    human_readable: false,
                                },
                            ],

                            // It's for debugging the compiler
                            dump_type_info: true,

                            output_path: None,
                            backend: Backend::Bytecode,  // doesn't matter
                            stop_after: CompileStage::Hir,
                            profile: Profile::Test,
                            optimization,
                        },
                    ],
                    id: run_id,
                }).map_err(|_| Error::ProcessError)?;
                unfinished_runs.insert(run_id);
                run_id += 1;

                // loop 1: generate hir of all files
                loop {
                    for (worker_id, worker) in workers.iter().enumerate() {
                        match worker.try_recv() {
                            Ok(msg) => match msg {
                                MessageToMain::FoundExternalModule {
                                    module_path,
                                    span,
                                } => {
                                    if !generated_hirs.contains(&module_path) {
                                        generated_hirs.insert(module_path.clone());
                                        let file_path = find_module_file(
                                            &module_path,
                                            span,
                                            "target",
                                            None,
                                        )?;
                                        workers[run_id % workers.len()].send(MessageToWorker::Run {
                                            commands: vec![Command::Compile {
                                                input_file_path: file_path,
                                                input_module_path: module_path,
                                                intermediate_dir: String::from("target"),
                                                emit_ir_options: vec![
                                                    EmitIrOption {
                                                        stage: CompileStage::Hir,
                                                        store: StoreIrAt::IntermediateDir,
                                                        human_readable: false,
                                                    },
                                                ],
                                                dump_type_info: true,
                                                output_path: None,
                                                backend: Backend::Bytecode,
                                                stop_after: CompileStage::Hir,
                                                profile: Profile::Test,
                                                optimization,
                                            }],
                                            id: run_id,
                                        }).map_err(|_| Error::ProcessError)?;
                                        unfinished_runs.insert(run_id);
                                        run_id += 1;
                                    }
                                },
                                MessageToMain::RunComplete { id } => {
                                    unfinished_runs.remove(&id);
                                },
                                MessageToMain::Error(e) => {
                                    return Err(e);
                                },
                            },
                            Err(mpsc::TryRecvError::Empty) => {},
                            Err(mpsc::TryRecvError::Disconnected) => {
                                return Err(Error::ProcessError);
                            },
                        }
                    }

                    if unfinished_runs.is_empty() {
                        break;
                    }

                    thread::sleep(Duration::from_millis(200));
                }

                workers[run_id % workers.len()].send(MessageToWorker::Run {
                    commands: vec![Command::InterHir {
                        modules: generated_hirs.iter().map(|module| module.to_string()).collect(),
                        intermediate_dir: String::from("target"),
                    }],
                    id: run_id,
                }).map_err(|_| Error::ProcessError)?;
                unfinished_runs.insert(run_id);
                run_id += 1;

                // loop 2: generate inter-hir map
                loop {
                    // TODO
                }

                Ok(())
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
    loop {
        for f in read_dir(".", false)? {
            if basename(&f)? == "sodigy.toml" {
                return Ok(());
            }
        }

        set_current_dir("..")?;
    }
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
            Command::Compile {
                input_file_path,
                input_module_path,
                intermediate_dir,
                emit_ir_options,
                dump_type_info,
                output_path,
                stop_after,
                backend,
                profile,
                optimization,
            } => {
                let bytes = std::fs::read(&input_file_path).map_err(
                    |e| Error::FileError(FileError::from_std(e, &input_file_path))
                )?;
                let file = File::register(
                    0,  // project_id
                    &input_file_path,
                    &input_module_path,
                    &intermediate_dir,
                )?;
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
                    cached_hir_session = Some(hir::Session::decode(&cached_data)?);
                }

                let hir_session = if let Some(mut hir_session) = cached_hir_session {
                    hir_session.intermediate_dir = intermediate_dir.clone();
                    hir_session
                } else {
                    let lex_session = sodigy_lex::lex(
                        file,
                        bytes,
                        intermediate_dir.clone(),
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

                    sodigy_hir::lower(parse_session)
                };

                emit_irs_if_has_to(
                    &hir_session,
                    &emit_ir_options,
                    CompileStage::Hir,
                    Some(content_hash),
                    &intermediate_dir,
                    &mut memory,
                )?;
                hir_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                if let CompileStage::Hir = stop_after {
                    continue;
                }

                let external_names = hir_session.uses.iter().map(
                    |hir::Use { root: IdentWithOrigin { id, origin, span, .. }, .. }| (*id, *origin, *span)
                ).filter(
                    |(_, origin, _)| matches!(origin, NameOrigin::External)
                ).map(
                    |(id, _, span)| (id, span)
                ).collect::<HashSet<_>>();
                let std_name = intern_string(b"std", &intermediate_dir)?;

                for (name, span) in external_names.iter() {
                    if *name != std_name {
                        tx_to_main.send(MessageToMain::FoundExternalModule {
                            module_path: todo!(),
                            span: *span,
                        }).map_err(|_| Error::ProcessError)?;
                    }
                }

                if let CompileStage::InterHir = stop_after {
                    continue;
                }

                // TODO: inter-file hir analysis (name-resolution and applying type-aliases)
                // There are 3 types of files: current_compiling_file, std, and dependencies
                // We have hir of current_compiling_file, and other processes might have created
                // hir of other types of files and saved them on disk.
                // In order for name-resolution, we need a giant map that has names and type signatures
                // of everything in every file (we don't need expressions).
                // The giant map can be reused.
                // So, a process first creates the giant map, and each process uses the giant map
                // for name-resolution in their hir.

                let mir_session = sodigy_mir::lower(hir_session);
                emit_irs_if_has_to(
                    &mir_session,
                    &emit_ir_options,
                    CompileStage::Mir,
                    None,
                    &intermediate_dir,
                    &mut memory,
                )?;
                mir_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                if let CompileStage::Mir = stop_after {
                    continue;
                }

                // TODO: dump type_solver
                let (mut mir_session, type_solver) = sodigy_mir_type::solve(mir_session);
                mir_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                if dump_type_info {
                    sodigy_mir_type::dump(&mut mir_session, &type_solver);
                }

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

                let executable = lir_session.into_executable(optimization == Optimization::None);

                let result = match backend {
                    Backend::Python => sodigy_backend::python_code_gen(
                        &executable,
                        &sodigy_backend::CodeGenConfig {
                            intermediate_dir: intermediate_dir.clone(),
                            label_help_comment: true,
                            mode: profile.into(),
                        },
                    )?,
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
            Command::InterHir {
                modules,
                intermediate_dir,
            } => {
                let hir_ids = sodigy_file::get_content_hashes(
                    0,  // project_id
                    &modules,
                    &intermediate_dir,
                )?;
                let mut inter_hir_session = sodigy_inter_hir::Session::new(&intermediate_dir);

                for hir_id in hir_ids.iter() {
                    let hir_session_bytes = get_cached_ir(
                        &intermediate_dir,
                        CompileStage::Hir,
                        Some(*hir_id),
                    )?;

                    let mut hir_session = match hir_session_bytes.map(|bytes| sodigy_hir::Session::decode(&bytes)) {
                        Some(Ok(session)) => session,

                        // TODO: It's kinda ICE, but there's no interface for ICE yet
                        _ => todo!(),
                    };

                    hir_session.intermediate_dir = intermediate_dir.clone();
                    inter_hir_session.ingest(hir_session);
                }

                emit_irs_if_has_to(
                    &inter_hir_session,
                    &[EmitIrOption {
                        stage: CompileStage::InterHir,
                        store: StoreIrAt::IntermediateDir,
                        human_readable: false,
                    }],
                    CompileStage::InterHir,
                    None,
                    &intermediate_dir,
                    &mut memory,
                )?;
                inter_hir_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;
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
                let bytecodes = Executable::decode(&bytecodes_bytes)?;

                match profile {
                    Profile::Test => {
                        let mut failed = false;

                        // TODO: it has to capture stderr and stdout
                        for (id, name) in bytecodes.asserts.iter() {
                            if let Err(_) = sodigy_backend::interpret(
                                &bytecodes.bytecodes,
                                *id,
                            ) {
                                println!("{name}: \x1b[31mFail\x1b[0m");
                                failed = true;
                            }

                            else {
                                println!("{name}: \x1b[32mPass\x1b[0m");
                            }
                        }

                        if failed {
                            return Err(Error::TestError);
                        }
                    },
                    _ => {
                        if let Err(e) = sodigy_backend::interpret(
                            &bytecodes.bytecodes,
                            bytecodes.main_func.unwrap(),
                        ) {
                            // what else do we do here?
                            panic!("TODO: {e:?}")
                        }
                    },
                }
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
    }

    // TODO: We only a few of these dirs
    for stage in CompileStage::all() {
        let stage_ir_dir = join(
            &ir_dir,
            &format!("{stage:?}").to_lowercase(),
        )?;

        if !exists(&stage_ir_dir) {
            create_dir_all(&stage_ir_dir)?;
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
                write_bytes(&s, content, WriteMode::CreateOrTruncate)?;
            },
            StoreIrAt::Memory => {
                *memory = Some(content.to_vec());
            },
            StoreIrAt::IntermediateDir => {
                let path = if let Some(content_hash) = content_hash {
                    join4(
                        intermediate_dir,
                        "irs",
                        &format!("{finished_stage:?}").to_lowercase(),
                        &format!("{content_hash:x}{ext}"),
                    )?
                } else {
                    join3(
                        intermediate_dir,
                        "irs",
                        &format!("{finished_stage:?}{ext}").to_lowercase(),
                    )?
                };

                write_bytes(
                    &path,
                    content,
                    WriteMode::CreateOrTruncate,
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
    let path = if let Some(content_hash) = content_hash {
        join4(
            intermediate_dir,
            "irs",
            &format!("{stage:?}").to_lowercase(),
            &format!("{content_hash:x}"),
        )?
    } else {
        join3(
            intermediate_dir,
            "irs",
            &format!("{stage:?}").to_lowercase(),
        )?
    };

    if exists(&path) {
        Ok(Some(read_bytes(&path)?))
    }

    else {
        Ok(None)
    }
}

fn find_module_file(
    // It's always normalized.
    // "foo/bar"
    module: &str,

    // for error message
    span: Span,
    intermediate_dir: &str,
    error_note: Option<String>,
) -> Result<String, Error> {
    let candidate1 = format!("src/{module}.sdg");
    let candidate2 = format!("src/{module}/mod.sdg");

    let result = match (exists(&candidate1), exists(&candidate2)) {
        (true, true) => Err(SodigyErrorKind::MultipleModuleFiles { module: module.to_string() }),
        (false, false) => Err(SodigyErrorKind::ModuleFileNotFound { module: module.to_string() }),
        (true, false) => Ok(candidate1),
        (false, true) => Ok(candidate2),
    };

    match result {
        Ok(path) => Ok(path),
        Err(e) => {
            let dummy_session = DummySession {
                errors: vec![SodigyError {
                    kind: e,
                    spans: span.simple_error(),
                    note: error_note,
                }],
                warnings: vec![],
                intermediate_dir: intermediate_dir.to_string(),
            };
            dummy_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;
            unreachable!()
        },
    }
}
