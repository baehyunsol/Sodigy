use super::*;
use sodigy_err::SodigyError;
use sodigy_files::{get_all_sdg, global_file_session};
use sodigy_lex::{lex, LexSession};
use sodigy_span::SpanPoint;
use sodigy_parse::{from_tokens, ParseSession};

#[test]
fn ast_test() {
    let g = unsafe { global_file_session() };

    for path in get_all_sdg("../../samples", true, "sdg").unwrap() {
        let mut lex_session = LexSession::new();
        let f = g.register_file(&path.to_string());
        let content = g.get_file_content(f).unwrap();

        lex(&content, 0, SpanPoint::at_file(f, 0), &mut lex_session).unwrap();

        let mut parse_session = ParseSession::from_lex_session(&lex_session);
        from_tokens(lex_session.get_tokens(), &mut parse_session, &mut LexSession::new()).unwrap();

        let mut ast_session = AstSession::from_parse_session(&parse_session);
        let mut tokens = parse_session.get_tokens().to_vec();
        let mut tokens = Tokens::from_vec(&mut tokens);

        if let Err(()) = parse_stmts(&mut tokens, &mut ast_session) {
            for error in ast_session.get_errors() {
                println!("{}\n\n", error.render_error());
            }
        }
    }
}
