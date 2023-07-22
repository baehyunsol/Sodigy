pub use sdg_ast::{
    AST, GlobalParseSession, LocalParseSession, SodigyError,
    parse_file,
};

// What I've just found out: `#[cfg(test)]` works only inside a crate
#[cfg(test)]
mod tests {
    use super::*;
    use sdg_fs::read_bytes;

    // TODOs
    // 1. initialize global session instead of a local one
    // 2. initialize a local one from the global one
    // 3. register `tests.sdg`: don't use `#[cfg(test)]` with sessions
    #[test]
    fn parse_tests() {
        let mut session = LocalParseSession::new();
        let input = read_bytes("./tests.sdg").unwrap();
        session.set_input(input.clone());

        match parse_file(&input, &mut session) {
            Ok(ast) => {
                // TODO: run tests
            },
            Err(e) => panic!("\n\n{}\n\n", e.render_err(&session)),
        }
    }
}