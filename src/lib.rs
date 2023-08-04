pub use sdg_ast::{
    AST, GlobalParseSession, LocalParseSession, SodigyError,
    parse_file,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tests() {
        let mut session = LocalParseSession::new();
        session.set_input("./tests/syntax.sdg").map_err(|e| e.render_err(&session)).unwrap();
        let input = session.get_curr_file_content().to_vec();

        match parse_file(&input, &mut session) {
            Ok(ast) => {
                println!("{}", ast.dump(&session));
                if !session.has_no_warning() {
                    panic!("\n\n{}\n\n", session.render_warnings());
                }

                // TODO: run tests
            },
            Err(_) => panic!("\n\n{}\n\n", session.render_err()),
        }
    }
}
