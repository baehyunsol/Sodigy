use sodigy::{
    Backend,
    Command,
    FileOrMemory,
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
use sodigy_session::Session;

fn main() -> Result<(), ()> {
    let args = std::env::args().collect::<Vec<_>>();
    let mut compile_result = None;

    match parse_args(&args) {
        Ok(commands) => {
            for command in commands.into_iter() {
                match command {
                    Command::InitIrDir {
                        intermediate_dir,
                    } => if let Err(e) = init_ir_dir(&intermediate_dir) {
                        eprintln!("{e:?}");
                        return Err(());
                    },
                    // TODO: there are too many unwraps
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
                    } => {
                        let bytes = match std::fs::read(&input_path) {
                            Ok(bytes) => bytes,
                            _ => todo!(),
                        };
                        let file = File::register(
                            0,  // project_id
                            &input_path,

                            // TODO: It's `normalized_path`, but I'm too lazy to normalize the path.
                            &input_path,

                            &intermediate_dir,
                        ).unwrap();

                        let lex_session = sodigy_lex::lex(
                            file,
                            bytes,
                            intermediate_dir.clone(),
                        );
                        lex_session.continue_or_dump_errors()?;

                        if emit_irs {
                            write_string(
                                &join3(
                                    &intermediate_dir,
                                    "irs",
                                    "tokens.rs",
                                ).unwrap(),
                                &prettify(&format!("{:?}", lex_session.tokens)),
                                WriteMode::CreateOrTruncate,
                            ).unwrap();
                        }

                        let parse_session = sodigy_parse::parse(lex_session);
                        parse_session.continue_or_dump_errors()?;

                        if emit_irs {
                            write_string(
                                &join3(
                                    &intermediate_dir,
                                    "irs",
                                    "ast.rs",
                                ).unwrap(),
                                &prettify(&format!("{:?}", parse_session.ast)),
                                WriteMode::CreateOrTruncate,
                            ).unwrap();
                        }

                        let hir_session = sodigy_hir::lower(parse_session);
                        hir_session.continue_or_dump_errors()?;

                        // TODO: inter-file hir analysis (name-resolution and applying type-aliases)

                        if emit_irs {
                            write_string(
                                &join3(
                                    &intermediate_dir,
                                    "irs",
                                    "hir.rs",
                                ).unwrap(),
                                &prettify(&format!(
                                    "{} lets: {:?}, funcs: {:?}, asserts: {:?} {}",
                                    "{",
                                    hir_session.lets,
                                    hir_session.funcs,
                                    hir_session.asserts,
                                    "}",
                                )),
                                WriteMode::CreateOrTruncate,
                            ).unwrap();
                        }

                        let mir_session = sodigy_mir::lower(hir_session);
                        mir_session.continue_or_dump_errors()?;

                        if emit_irs {
                            write_string(
                                &join3(
                                    &intermediate_dir,
                                    "irs",
                                    "mir.rs",
                                ).unwrap(),
                                &prettify(&format!(
                                    "{} lets: {:?}, funcs: {:?}, asserts: {:?} {}",
                                    "{",
                                    mir_session.lets,
                                    mir_session.funcs,
                                    mir_session.asserts,
                                    "}",
                                )),
                                WriteMode::CreateOrTruncate,
                            ).unwrap();
                        }

                        let (mut mir_session, solver) = sodigy_mir_type::solve(mir_session);
                        mir_session.continue_or_dump_errors()?;

                        if dump_type_info {
                            sodigy_mir_type::dump(&mut mir_session, &solver);
                        }

                        let mut lir_session = sodigy_lir::lower(mir_session);
                        lir_session.continue_or_dump_errors()?;

                        if emit_irs {
                            write_string(
                                &join3(
                                    &intermediate_dir,
                                    "irs",
                                    "lir.rs",
                                ).unwrap(),
                                &prettify(&format!(
                                    "{} lets: {:?}, funcs: {:?}, asserts: {:?} {}",
                                    "{",
                                    lir_session.lets,
                                    lir_session.funcs,
                                    lir_session.asserts,
                                    "}",
                                )),
                                WriteMode::CreateOrTruncate,
                            ).unwrap();
                        }

                        let executable = lir_session.into_executable(profile != Profile::Release);
                        lir_session.dump_warnings();

                        match output_path {
                            FileOrMemory::File(path) => {
                                let result = match backend {
                                    Backend::Python => sodigy_backend::python_code_gen(
                                        &executable,
                                        &sodigy_backend::CodeGenConfig {
                                            intermediate_dir,
                                            label_help_comment: true,
                                            mode: profile.into(),
                                        },
                                    ).unwrap(),
                                    _ => todo!(),
                                };
                                write_bytes(
                                    &path,
                                    &result,
                                    WriteMode::CreateOrTruncate,
                                ).unwrap();
                            },
                            FileOrMemory::Memory => match backend {
                                Backend::Bytecode => {
                                    compile_result = Some(executable);
                                },
                                _ => unreachable!(),
                            },
                        }
                    },
                    Command::Interpret {
                        executable_path,
                        profile,
                    } => {
                        let executable = match executable_path {
                            FileOrMemory::File(path) => todo!(),
                            FileOrMemory::Memory => compile_result.clone().unwrap(),
                        };

                        match profile {
                            Profile::Test => {
                                let mut failures = vec![];

                                for (id, name) in executable.asserts.iter() {
                                    if let Err(e) = sodigy_backend::interpret(
                                        &executable.bytecodes,
                                        *id,
                                    ) {
                                        failures.push(name.to_string());
                                    }
                                }

                                for failure in failures.iter() {
                                    todo!()
                                }

                                if !failures.is_empty() {
                                    return Err(());
                                }
                            },
                            _ => {
                                if let Err(e) = sodigy_backend::interpret(
                                    &executable.bytecodes,
                                    executable.main_func.unwrap(),
                                ) {
                                    // what else do we do here?
                                    return Err(());
                                }
                            },
                        }
                    },
                    Command::Help(doc) => {},
                }
            }

            Ok(())
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
            Err(())
        },
    }
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
