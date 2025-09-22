// gara test code
fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let bytes = std::fs::read(&args[1]).unwrap();
    let mut lex_session = sodigy_lex::LexSession::gara_init(bytes);
    lex_session.lex();
    // println!("{:?}", lex_session.tokens);
    let mut parse_session = sodigy_parse::ParseSession::from_lex_session(&lex_session);
    let ast = parse_session.parse().unwrap();
    println!("{ast:?}");
}
