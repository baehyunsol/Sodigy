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
    write_bytes,
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
                            "todo",  // normalized_path
                            &intermediate_dir,
                        ).unwrap();

                        let lex_session = sodigy_lex::lex(
                            file,
                            bytes,
                            intermediate_dir.clone(),
                        );
                        lex_session.continue_or_dump_errors()?;
                        let parse_session = sodigy_parse::parse(lex_session);
                        parse_session.continue_or_dump_errors()?;
                        let hir_session = sodigy_hir::lower(parse_session);
                        hir_session.continue_or_dump_errors()?;

                        // TODO: inter-file hir analysis (name-resolution and applying type-aliases)

                        let mir_session = sodigy_mir::lower(hir_session);
                        mir_session.continue_or_dump_errors()?;
                        let mir_session = sodigy_mir_type::solve(mir_session);
                        mir_session.continue_or_dump_errors()?;
                        let lir_session = sodigy_lir::lower(mir_session);
                        lir_session.continue_or_dump_errors()?;
                        let bytecode = lir_session.into_labeled_bytecode();
                        lir_session.dump_warnings();

                        match output_path {
                            FileOrMemory::File(path) => {
                                let result = match backend {
                                    Backend::Python => sodigy_backend::python_code_gen(
                                        &bytecode,
                                        &lir_session,
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
                                    compile_result = Some(bytecode);
                                },
                                _ => unreachable!(),
                            },
                        }
                    },
                    Command::Interpret {
                        bytecode_path,
                        profile,
                    } => match bytecode_path {
                        FileOrMemory::File(path) => todo!(),
                        FileOrMemory::Memory => {
                            let compile_result = compile_result.as_ref().unwrap();

                            match profile {
                                Profile::Test => {},
                                _ => {},
                            }
                        },
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
    let intern_str_map_dir = join(&intermediate_dir, "str")?;
    let intern_num_map_dir = join(&intermediate_dir, "num")?;

    if !exists(&intern_str_map_dir) {
        create_dir_all(&intern_str_map_dir)?;
    }

    if !exists(&intern_num_map_dir) {
        create_dir_all(&intern_num_map_dir)?;
    }

    File::clear_cache(0 /* project id */, intermediate_dir)?;
    Ok(())
}

fn prettify(s: &str) -> String {
    let mut c = hgp::Context::new(s.as_bytes().to_vec());
    c.step_all();
    String::from_utf8_lossy(c.output()).to_string()
}
