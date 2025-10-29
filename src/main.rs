use sodigy::{
    Backend,
    CliCommand,
    Command,
    Error,
    IrKind,
    IrStore,
    Optimization,
    Profile,
    parse_args,
};
use sodigy_file::File;
use sodigy_fs_api::{
    FileError,
    WriteMode,
    create_dir_all,
    exists,
    join,
    join3,
    write_bytes,
    write_string,
};
use sodigy_hir as hir;
use sodigy_name_analysis::{IdentWithOrigin, NameOrigin};
use sodigy_session::Session;
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
            CliCommand::Test {
                optimization,
                jobs,
            } => {
                let workers = worker::init_workers(jobs);

                // This is the main worker. It'll run the VM.
                workers[0].send(MessageToWorker::Run(vec![
                    Command::InitIrDir {
                        intermediate_dir: todo!(),
                    },
                    Command::Compile {
                        input_path: todo!(),
                        input_kind: IrKind::Code,
                        intermediate_dir: todo!(),
                        reuse_ir: true,

                        // These 2 are for debugging the compiler
                        emit_irs: true,
                        dump_type_info: true,

                        output_path: IrStore::BytecodesOnMemory,
                        output_kind: IrKind::TranspiledCode,
                        backend: Backend::Bytecode,
                        profile: Profile::Test,
                        optimization,
                    },
                    Command::Interpret {
                        bytecodes_path: IrStore::BytecodesOnMemory,
                        profile: Profile::Test,
                    },
                ])).unwrap();
                
                for (worker_id, worker) in workers.iter().enumerate() {
                    match worker.try_recv() {
                        Ok(msg) => match msg {
                            MessageToMain::Error(e) => {
                                return Err(e);
                            },
                        },
                        Err(mpsc::TryRecvError::Empty) => {},
                        Err(mpsc::TryRecvError::Disconnected) => {
                            return Err(Error::ProcessError);
                        },
                    }

                    thread::sleep(Duration::from_millis(200));
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

pub fn run(commands: Vec<Command>) -> Result<(), Error> {
    let mut compile_result = None;

    for command in commands.into_iter() {
        match command {
            Command::InitIrDir {
                intermediate_dir,
            } => init_ir_dir(&intermediate_dir)?,
            // TODO: It assumes that `input_kind` is always `IrKind::Code` and
            //       `output_kind` is always `IrKind::TranspiledCode` because
            //       the other variants are not implemented yet.
            Command::Compile {
                input_path,
                input_kind,
                intermediate_dir,
                reuse_ir,
                emit_irs,
                dump_type_info,
                output_path,
                output_kind,
                backend,
                profile,
                optimization,
            } => {
                let bytes = std::fs::read(&input_path).map_err(
                    |e| Error::FileError(FileError::from_std(e, &input_path))
                )?;
                let file = File::register(
                    0,  // project_id
                    &input_path,

                    // TODO: It's `normalized_path`, but I'm too lazy to normalize the path.
                    &input_path,

                    &intermediate_dir,
                )?;

                let lex_session = sodigy_lex::lex(
                    file,
                    bytes,
                    intermediate_dir.clone(),
                );
                lex_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                if emit_irs {
                    write_string(
                        &join3(
                            &intermediate_dir,
                            "irs",
                            "tokens.rs",
                        )?,
                        &prettify(&format!("{:?}", lex_session.tokens)),
                        WriteMode::CreateOrTruncate,
                    )?;
                }

                let parse_session = sodigy_parse::parse(lex_session);
                parse_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                if emit_irs {
                    write_string(
                        &join3(
                            &intermediate_dir,
                            "irs",
                            "ast.rs",
                        )?,
                        &prettify(&format!("{:?}", parse_session.ast)),
                        WriteMode::CreateOrTruncate,
                    )?;
                }

                let hir_session = sodigy_hir::lower(parse_session);
                hir_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                if emit_irs {
                    write_string(
                        &join3(
                            &intermediate_dir,
                            "irs",
                            "hir.rs",
                        )?,
                        &prettify(&format!(
                            "{} lets: {:?}, funcs: {:?}, asserts: {:?}, uses: {:?} {}",
                            "{",
                            hir_session.lets,
                            hir_session.funcs,
                            hir_session.asserts,
                            hir_session.uses,
                            "}",
                        )),
                        WriteMode::CreateOrTruncate,
                    )?;
                }

                let external_names = hir_session.uses.iter().map(
                    |hir::Use { root: IdentWithOrigin { id, origin, .. }, .. }| (*id, *origin)
                ).filter(
                    |(_, origin)| matches!(origin, NameOrigin::External)
                ).map(
                    |(id, _)| id
                ).collect::<HashSet<_>>();
                let std_name = intern_string(b"std", &intermediate_dir)?;

                for name in external_names.iter() {
                    if *name != std_name {
                        // TODO: create hir and load it
                        todo!()
                    }
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
                mir_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                if emit_irs {
                    write_string(
                        &join3(
                            &intermediate_dir,
                            "irs",
                            "mir.rs",
                        )?,
                        &prettify(&format!(
                            "{} lets: {:?}, funcs: {:?}, asserts: {:?} {}",
                            "{",
                            mir_session.lets,
                            mir_session.funcs,
                            mir_session.asserts,
                            "}",
                        )),
                        WriteMode::CreateOrTruncate,
                    )?;
                }

                let (mut mir_session, type_solver) = sodigy_mir_type::solve(mir_session);
                mir_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                if dump_type_info {
                    sodigy_mir_type::dump(&mut mir_session, &type_solver);
                }

                let mut lir_session = sodigy_lir::lower(mir_session);
                lir_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;

                if emit_irs {
                    write_string(
                        &join3(
                            &intermediate_dir,
                            "irs",
                            "lir.rs",
                        )?,
                        &prettify(&format!(
                            "{} lets: {:?}, funcs: {:?}, asserts: {:?} {}",
                            "{",
                            lir_session.lets,
                            lir_session.funcs,
                            lir_session.asserts,
                            "}",
                        )),
                        WriteMode::CreateOrTruncate,
                    )?;
                }

                let executable = lir_session.into_executable(optimization == Optimization::None);
                lir_session.dump_warnings();

                match output_path {
                    IrStore::File(path) => {
                        let result = match backend {
                            Backend::Python => sodigy_backend::python_code_gen(
                                &executable,
                                &sodigy_backend::CodeGenConfig {
                                    intermediate_dir,
                                    label_help_comment: true,
                                    mode: profile.into(),
                                },
                            )?,
                            _ => todo!(),
                        };
                        write_bytes(
                            &path,
                            &result,
                            WriteMode::CreateOrTruncate,
                        )?;
                    },
                    IrStore::BytecodesOnMemory => match backend {
                        Backend::Bytecode => {
                            compile_result = Some(executable);
                        },
                        _ => unreachable!(),
                    },
                    IrStore::IntermediateDir => todo!(),
                }
            },
            Command::Interpret {
                bytecodes_path,
                profile,
            } => {
                let bytecodes = match bytecodes_path {
                    IrStore::File(path) => todo!(),
                    IrStore::BytecodesOnMemory => compile_result.clone().unwrap(),
                    IrStore::IntermediateDir => todo!(),
                };

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

    File::clear_cache(0 /* project id */, intermediate_dir)?;
    Ok(())
}

fn prettify(s: &str) -> String {
    let mut c = hgp::Context::new(s.as_bytes().to_vec());
    c.step_all();
    String::from_utf8_lossy(c.output()).to_string()
}
