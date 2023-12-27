use super::*;
use sodigy_error::SodigyError;
use sodigy_files::{global_file_session, get_all_sdg, FileHash};
use sodigy_lex::{lex, LexSession};
use sodigy_span::SpanPoint;

#[test]
fn parse_test() {
    std::env::set_var("RUST_BACKTRACE", "FULL");

    let codes = vec![
        "\"Hello, world!\"",
        "b\"Hello, world!\"",
        "[a, b, c, d]",
        "3 + 3 = x;",
        "foo(a, b, c, [3, 4, 5]);  # this is a comment",
        "f\"\\{1} + \\{2} = \\{1 + 2}\"",
        "f\"{\\{1}} + {\\{2}} = {\\{1 + 2}}\"",
        "##! nested comment  ##! !## !## 3 + 4",
        "a.b.c.d",
        "people `name \"Bae\"",
        "\\{x, y, x + y}(10, 10)",
    ];
    let g = unsafe { global_file_session() };

    for code in codes.into_iter() {
        let mut lex_session = LexSession::new();
        let f = g.register_tmp_file(code.as_bytes().to_vec());
        let content = g.get_file_content(f).unwrap();

        test_runner(f, content, &mut lex_session);
    }

    for path in get_all_sdg("../../samples", true, "sdg").unwrap() {
        let mut lex_session = LexSession::new();
        let f = g.register_file(&path.to_string()).unwrap();
        let content = g.get_file_content(f).unwrap();

        test_runner(f, content, &mut lex_session);
    }
}

fn test_runner(f: FileHash, content: &[u8], lex_session: &mut LexSession) {
    if let Err(()) = lex(content, 0, SpanPoint::at_file(f, 0), lex_session) {
        panic!(
            "{}",
            lex_session.get_errors().iter().map(
                |e| e.render_error()
            ).collect::<Vec<String>>().join("\n\n"),
        );
    }

    println!("{}\n\n", lex_session.dump_tokens());

    let mut parse_session = ParseSession::from_lex_session(&lex_session);

    if let Err(()) = from_tokens(&lex_session.get_tokens(), &mut parse_session, &mut LexSession::new()) {
        panic!(
            "{}",
            lex_session.get_errors().iter().map(
                |e| e.render_error()
            ).chain(
                parse_session.get_errors().iter().map(
                    |e| e.render_error()
                )
            ).collect::<Vec<String>>().join("\n\n"),
        );
    }

    // round trip test

    let token_round_trip_test = parse_session.dump_tokens();

    lex_session.flush_tokens();
    parse_session.flush_tokens();

    let g = unsafe { global_file_session() };
    let f = g.register_tmp_file(token_round_trip_test.as_bytes().to_vec());

    if let Err(()) = lex(token_round_trip_test.as_bytes(), 0, SpanPoint::at_file(f, 0), lex_session) {
        panic!(
            "{}",
            lex_session.get_errors().iter().map(
                |e| e.render_error()
            ).chain(
                parse_session.get_errors().iter().map(
                    |e| e.render_error()
                )
            ).collect::<Vec<String>>().join("\n\n"),
        );
    }

    if let Err(()) = from_tokens(&lex_session.get_tokens().to_vec(), &mut parse_session, lex_session) {
        panic!(
            "{}",
            lex_session.get_errors().iter().map(
                |e| e.render_error()
            ).chain(
                parse_session.get_errors().iter().map(
                    |e| e.render_error()
                )
            ).collect::<Vec<String>>().join("\n\n"),
        );
    }

    let token_round_trip_result = parse_session.dump_tokens();

    assert_eq!(token_round_trip_test, token_round_trip_result);
}
