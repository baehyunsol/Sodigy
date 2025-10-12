use sodigy_error::{Error, ErrorLevel};
use sodigy_file::File;
use sodigy_fs_api::{
    WriteMode,
    create_dir_all,
    exists,
    remove_dir_all,
    write_bytes,
    write_string,
};

// gara test code
fn main() {
    if exists("sample/target/") {
        remove_dir_all("sample/target").unwrap();
    }

    create_dir_all("sample/target/").unwrap();
    create_dir_all("sample/target/intern/str/").unwrap();
    create_dir_all("sample/target/intern/num/").unwrap();
    write_bytes(
        "sample/target/intern/str/lock",
        b"",
        WriteMode::CreateOrTruncate,
    ).unwrap();
    write_bytes(
        "sample/target/intern/num/lock",
        b"",
        WriteMode::CreateOrTruncate,
    ).unwrap();

    let args = std::env::args().collect::<Vec<String>>();
    let bytes = std::fs::read(&args[1]).unwrap();
    let file = File::gara();

    let tokens = match sodigy_lex::lex_gara(bytes.clone(), "sample/target/intern/") {
        Ok(tokens) => tokens,
        Err(error) => {
            eprintln!("{}", render_errors(&args[1], &bytes, vec![error], "sample/target/intern/str/"));
            return;
        },
    };
    write_string(
        "sample/target/tokens.rs",
        &prettify(&format!("{tokens:?}")),
        WriteMode::CreateOrTruncate,
    ).unwrap();

    let ast_block = match sodigy_parse::parse(&tokens, file) {
        Ok(ast_block) => ast_block,
        Err(errors) => {
            eprintln!("{}", render_errors(&args[1], &bytes, errors, "sample/target/intern/str/"));
            return;
        },
    };
    write_string(
        "sample/target/ast.rs",
        &prettify(&format!("{ast_block:?}")),
        WriteMode::CreateOrTruncate,
    ).unwrap();

    let mut hir_session = sodigy_hir::Session::new("sample/target/intern/");

    let has_error = hir_session.lower(&ast_block).is_err();
    eprintln!("{}", render_errors(
        &args[1],
        &bytes,
        vec![
            hir_session.errors.clone(),
            hir_session.warnings.clone(),
        ].concat(),
        "sample/target/intern/str/",
    ));

    if has_error {
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
    eprintln!("{}", render_errors(
        &args[1],
        &bytes,
        vec![
            mir_session.errors.clone(),
            // mir_session.warnings.clone(),
        ].concat(),
        "sample/target/intern/str/",
    ));

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

    let lir_session = sodigy_lir::lower_mir(&mir_session);

    write_string(
        "sample/target/lir.rs",
        &prettify(&format!(
            "{}lets: {:?}, funcs: {:?}{}",
            "{",
            lir_session.lets,
            lir_session.funcs,
            "}",
        )),
        WriteMode::CreateOrTruncate,
    ).unwrap();
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

        buffer.push(format!(
            "{title}: {}\n{}\n\n",
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
