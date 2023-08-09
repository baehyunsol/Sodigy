pub use sdg_ast::{
    AST, GlobalParseSession, LocalParseSession, SodigyError,
    parse_file,
};

pub use sdg_inter_mod::{
    InterModuleContext,
    dump_module,
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
                // ast test
                // println!("{}", ast.dump(&mut session));

                // inter_mod test
                // let mut ctxt = InterModuleContext::new();
                // ctxt.collect_ast(&ast);
                // panic!("{}", dump_module(&ctxt.namespace, &session));

                if !session.has_no_warning() {
                    panic!("\n\n{}\n\n", session.render_warnings());
                }

                // TODO: run tests
            },
            Err(_) => panic!("\n\n{}\n\n", session.render_err()),
        }
    }
}
