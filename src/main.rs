use sodigy::{Command, parse_args};
use sodigy_error::{Error, ErrorLevel};
use sodigy_file::File;
use sodigy_fs_api::{
    WriteMode,
    create_dir_all,
    exists,
    join,
    remove_dir_all,
    write_bytes,
    write_string,
};
use sodigy_string::unintern_string;
use std::collections::HashMap;

fn main() -> Result<(), ()> {
    let args = std::env::args().collect::<Vec<_>>();

    match parse_args(&args) {
        Ok(commands) => {
            for command in commands.into_iter() {
                match command {
                    // TODO: there are too many `unwrap`s
                    // TODO: each pass looks slightly different
                    Command::Compile {
                        input_path,
                        input_kind,
                        intermediate_dir,
                        output_path,
                        output_kind,
                        backend,
                        profile,
                    } => {
                        let intern_str_map_dir = join(&intermediate_dir, "str").unwrap();
                        let intern_num_map_dir = join(&intermediate_dir, "num").unwrap();

                        if !exists(&intern_str_map_dir) {
                            create_dir_all(&intern_str_map_dir).unwrap();
                        }

                        if !exists(&intern_num_map_dir) {
                            create_dir_all(&intern_num_map_dir).unwrap();
                        }

                        write_bytes(
                            &join(&intern_str_map_dir, "lock").unwrap(),
                            b"",
                            WriteMode::CreateOrTruncate,
                        ).unwrap();

                        write_bytes(
                            &join(&intern_num_map_dir, "lock").unwrap(),
                            b"",
                            WriteMode::CreateOrTruncate,
                        ).unwrap();

                        let bytes = match std::fs::read(&input_path) {
                            Ok(bytes) => bytes,
                            _ => todo!(),
                        };
                        let file = File::Single;

                        let tokens = match sodigy_lex::lex(
                            file,
                            bytes.clone(),  // TODO: don't clone this
                            &intern_str_map_dir,
                        ) {
                            Ok(tokens) => tokens,
                            Err(error) => {
                                eprintln!("{}", render_errors(&input_path, &bytes, vec![error], &intern_str_map_dir));
                                return Err(());
                            },
                        };

                        let ast = match sodigy_parse::parse(&tokens, file) {
                            Ok(ast) => ast,
                            Err(errors) => {
                                eprintln!("{}", render_errors(&input_path, &bytes, errors, &intern_str_map_dir));
                                return Err(());
                            },
                        };

                        let mut hir_session = sodigy_hir::Session::new(&intern_str_map_dir);
                        let has_error = hir_session.lower(&ast).is_err();
                        eprintln!("{}", render_errors(
                            &args[1],
                            &bytes,
                            vec![
                                hir_session.errors.clone(),
                                hir_session.warnings.clone(),
                            ].concat(),
                            &intern_str_map_dir,
                        ));

                        if has_error {
                            return Err(());
                        }

                        // TODO: inter-file hir analysis

                        let mir_session = sodigy_mir::lower(&hir_session);
                        eprintln!("{}", render_errors(
                            &args[1],
                            &bytes,
                            vec![
                                mir_session.errors.clone(),
                                // mir_session.warnings.clone(),
                            ].concat(),
                            &intern_str_map_dir,
                        ));

                        if !mir_session.errors.is_empty() {
                            return Err(());
                        }

                        let mut lir_session = sodigy_lir::lower_mir(&mir_session);
                        lir_session.make_labels_static();
                        let bytecode = lir_session.into_labeled_bytecode();

                        sodigy_backend::python_code_gen(
                            &output_path,
                            &bytecode,
                            &lir_session,
                            &sodigy_backend::CodeGenConfig {
                                intern_str_map_dir,
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

fn prettify(s: &str) -> String {
    let mut c = hgp::Context::new(s.as_bytes().to_vec());
    c.step_all();
    String::from_utf8_lossy(c.output()).to_string()
}

fn render_errors(
    file_name: &str,
    bytes: &[u8],
    mut errors: Vec<Error>,
    intern_str_map_dir: &str,
) -> String {
    errors.sort_by_key(|e| (e.span, e.extra_span));
    // warnings come before errors
    errors.sort_by_key(
        |e| match ErrorLevel::from_error_kind(&e.kind) {
            ErrorLevel::Warning => 0,
            ErrorLevel::Error => 1,
        }
    );
    let mut buffer = vec![];

    for error in errors.iter() {
        let level = ErrorLevel::from_error_kind(&error.kind);
        let title = match level {
            ErrorLevel::Warning => level.color().render_fg("warning"),
            ErrorLevel::Error => level.color().render_fg("error"),
        };
        let note = if let Some(message) = &error.extra_message {
            format!("\nnote: {message}")
        } else {
            String::new()
        };

        buffer.push(format!(
            "{title}: {}{note}\n{}\n\n",
            error.kind.render(intern_str_map_dir),
            sodigy_span::render_span(
                file_name,
                bytes,
                error.span,
                error.extra_span,
                sodigy_span::RenderSpanOption {
                    max_width: 88,
                    max_height: 10,
                    render_source: true,
                    color: Some(sodigy_span::ColorOption {
                        primary: level.color(),
                        secondary: sodigy_span::Color::Green,
                    }),
                },
            ),
        ));
    }

    buffer.concat()
}
