pub use sdg_ast::{
    AST, GlobalParseSession, LocalParseSession, SodigyError,
    parse_file,
};

// What I've just found out: `#[cfg(test)]` works only inside a crate
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tests() {
        let mut session = LocalParseSession::new();
        session.set_input("./tests.sdg").map_err(|_| ()).unwrap();
        let input = session.get_curr_file_content().to_vec();

        match parse_file(&input, &mut session) {
            Ok(ast) => {
                if !session.has_no_warning() {
                    panic!("\n\n{}\n\n", session.render_warnings());
                }

                // TODO: run tests
            },
            Err(_) => panic!("\n\n{}\n\n", session.render_err()),
        }
    }
}
