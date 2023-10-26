use sodigy_ast::{parse_stmts, AstSession, Tokens};
use sodigy_err::SodigyError;
use sodigy_files::{get_all_sdg, global_file_session};
use sodigy_lex::{lex, lex_flex, LexSession};
use sodigy_parse::{from_tokens, ParseSession};
use sodigy_span::SpanPoint;

fn main() {
    let file_session = unsafe { global_file_session() };

    for file in get_all_sdg(
        "./samples/err", false, "in"
    ).unwrap().iter().chain(
        get_all_sdg("./samples", true, "sdg").unwrap().iter()
    ) {
        let file_hash = file_session.register_file(&file);
        run(&file_session.get_file_content(file_hash).unwrap(), file_hash);
    }
}

fn run(input: &[u8], file: u64) {
    let mut lex_session = LexSession::new();

    if let Err(()) = lex_flex!(input, 0, SpanPoint::at_file(file, 0), &mut lex_session) {
        for error in lex_session.get_errors() {
            println!("{}\n\n", error.render_error());
        }

        return;
    }

    let mut parse_session = ParseSession::from_lex_session(&lex_session);
    let tokens = lex_session.get_tokens();
    let mut new_lex_session = LexSession::new();

    if let Err(()) = from_tokens(tokens, &mut parse_session, &mut new_lex_session) {
        for error in parse_session.get_errors() {
            println!("{}\n\n", error.render_error());
        }

        for error in new_lex_session.get_errors() {
            println!("{}\n\n", error.render_error());
        }

        return;
    };

    let mut ast_session = AstSession::from_parse_session(&parse_session);
    let mut tokens = parse_session.get_tokens().to_vec();
    let mut tokens = Tokens::from_vec(&mut tokens);

    if let Err(()) = parse_stmts(&mut tokens, &mut ast_session) {
        for error in ast_session.get_errors() {
            println!("{}\n\n", error.render_error());
        }

        return;
    }
}
