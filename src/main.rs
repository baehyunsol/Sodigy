use sodigy::{Command, FileOrMemory, parse_args};
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

    match parse_args(&args) {
        Ok(commands) => {
            for command in commands.into_iter() {
                match command {
                    Command::InitIrDir {
                        intermediate_dir,
                    } => if let Err(e) = init_ir_dir(&intermediate_dir) {
                        return Err(());
                    },
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
                        // FIXME: the current implementation can only compile single-file projects.
                        let file = File::Single;

                        // TODO: if a session is erroneous, dump errors and quit
                        // TODO: always dump warnings
                        let lex_session = sodigy_lex::lex(
                            file,
                            bytes,
                            intermediate_dir.clone(),
                        );
                        lex_session.error_or_continue()?;
                        let parse_session = sodigy_parse::parse(lex_session);
                        parse_session.error_or_continue()?;
                        let hir_session = sodigy_hir::lower(parse_session);
                        hir_session.error_or_continue()?;
                        // TODO: inter-file hir analysis (name-resolution)
                        let mir_session = sodigy_mir::lower(hir_session);
                        mir_session.error_or_continue()?;
                        // TODO: inter-file mir analysis (type-check)
                        let lir_session = sodigy_lir::lower(mir_session);
                        lir_session.error_or_continue()?;
                        let bytecode = lir_session.into_labeled_bytecode();

                        let FileOrMemory::File(output_path) = output_path else { unreachable!() };
                        sodigy_backend::python_code_gen(
                            &output_path,
                            &bytecode,
                            &lir_session,
                            &sodigy_backend::CodeGenConfig {
                                intermediate_dir,
                                label_help_comment: true,
                                mode: profile.into(),
                            },
                        ).unwrap();
                    },
                    Command::Interpret {
                        bytecode_path,
                    } => {},
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

    write_bytes(
        &join(&intern_str_map_dir, "lock")?,
        b"",
        WriteMode::CreateOrTruncate,
    )?;

    write_bytes(
        &join(&intern_num_map_dir, "lock")?,
        b"",
        WriteMode::CreateOrTruncate,
    )?;
    Ok(())
}

fn prettify(s: &str) -> String {
    let mut c = hgp::Context::new(s.as_bytes().to_vec());
    c.step_all();
    String::from_utf8_lossy(c.output()).to_string()
}
