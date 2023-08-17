pub use sdg_ast::{
    AST, GlobalParseSession, LocalParseSession, SodigyError,
    parse_file, parse_files,
};

pub use sdg_inter_mod::{
    InterModuleContext,
    dump_module,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() {
        let mut session = LocalParseSession::new();
        match parse_files("./tests/main.sdg".into(), &mut session) {
            Ok(asts) => {
                if !session.has_no_warning() {
                    panic!("\n\n{}\n\n", session.render_warnings());
                }

                for ast in asts.into_iter() {
                    // ast test
                    // println!("{}", ast.dump(&mut session));

                    // inter_mod test
                    // let mut ctxt = InterModuleContext::new();
                    // ctxt.collect_ast(&ast);
                    // panic!("{}", dump_module(&ctxt.namespace, &session));

                    // TODO: run tests
                }
            }
            Err(_) => panic!("\n\n{}\n\n", session.render_err()),
        }
    }
}
