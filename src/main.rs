use ragit_fs::{
    WriteMode,
    create_dir_all,
    exists,
    remove_dir_all,
    write_string,
};
use sodigy_file::File;

// gara test code
fn main() {
    if exists("sample/target/") {
        remove_dir_all("sample/target").unwrap();
    }

    create_dir_all("sample/target/").unwrap();

    let args = std::env::args().collect::<Vec<String>>();
    let bytes = std::fs::read(&args[1]).unwrap();
    let file = File::gara();

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

    let ast_block = match sodigy_parse::parse(&tokens, file) {
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
    write_string(
        "sample/target/ast.rs",
        &prettify(&format!("{ast_block:?}")),
        WriteMode::CreateOrTruncate,
    ).unwrap();

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

    write_string(
        "sample/target/hir.rs",
        &prettify(&format!(
            "{}lets: {:?}, funcs: {:?}{}",
            "{",
            hir_session.lets,
            hir_session.funcs,
            "}",
        )),
        WriteMode::CreateOrTruncate,
    ).unwrap();

    // TODO: inter-file hir analysis

    let mir_session = sodigy_mir::lower(&hir_session);

    write_string(
        "sample/target/mir.rs",
        &prettify(&format!(
            "{}lets: {:?}, funcs: {:?}{}",
            "{",
            mir_session.lets,
            mir_session.funcs,
            "}",
        )),
        WriteMode::CreateOrTruncate,
    ).unwrap();

    let value = sodigy_mir_eval::eval_main(&mir_session).unwrap();
    println!("{value:?}");
}

fn prettify(s: &str) -> String {
    let mut c = hgp::Context::new(s.as_bytes().to_vec());
    c.step_all();
    String::from_utf8_lossy(c.output()).to_string()
}
