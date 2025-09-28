// gara test code
fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let bytes = std::fs::read(&args[1]).unwrap();

    let tokens = match sodigy_lex::lex_gara(bytes.clone()) {
        Ok(tokens) => tokens,
        Err(errors) => {
            for error in errors.iter() {
                eprintln!(
                    "{:?}\n{}\n",
                    error.kind,
                    sodigy_error::render_span(
                        &bytes,
                        error.span,
                        error.extra_span,
                        sodigy_error::RenderSpanOption {
                            max_width: 88,
                            max_height: 10,
                            color: true,
                        },
                    ),
                );
            }

            return;
        },
    };
    // println!("{tokens:?}");

    let ast_block = match sodigy_parse::parse(&tokens) {
        Ok(ast_block) => ast_block,
        Err(errors) => {
            for error in errors.iter() {
                eprintln!(
                    "{:?}\n{}\n",
                    error.kind,
                    sodigy_error::render_span(
                        &bytes,
                        error.span,
                        error.extra_span,
                        sodigy_error::RenderSpanOption {
                            max_width: 88,
                            max_height: 10,
                            color: true,
                        },
                    ),
                );
            }

            return;
        },
    };
    // println!("{ast_block:?}");

    let mut hir_session = sodigy_hir::Session::new();

    if let Err(()) = hir_session.lower(&ast_block) {
        for error in hir_session.errors.iter() {
            eprintln!(
                "{:?}\n{}\n",
                error.kind,
                sodigy_error::render_span(
                    &bytes,
                    error.span,
                    error.extra_span,
                    sodigy_error::RenderSpanOption {
                        max_width: 88,
                        max_height: 10,
                        color: true,
                    },
                ),
            );
        }

        return;
    }

    // TODO: inter-file hir analysis

    let mut mir_session = sodigy_mir::Session::new();
    let mir_block = mir_session.lower(&hir_block).unwrap();
    // println!("{mir_block:?}");
}
