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
        let mut inter_mod_ctxt = InterModuleContext::new(&mut session);
        let mut ty_ctxt = sdg_interpret::TypeCkCtxt::new();

        match parse_files("./tests/main.sdg".into(), &mut session) {
            Ok(asts) => {
                for ast in asts.into_iter() {
                    inter_mod_ctxt.collect_ast(&ast);

                    // ast test
                    println!("{}", ast.dump(&mut session));

                    sdg_interpret::type_check_ast(&ast, &mut session, &inter_mod_ctxt, &mut ty_ctxt).unwrap();

                    // TODO: run tests
                }

                // ctxt test
                // println!("{}", inter_mod_ctxt.dump(&mut session));

                if !session.has_no_warning() {
                    panic!("\n\n{}\n\n", session.render_warnings());
                }
            }
            Err(_) => panic!("\n\n{}\n\n", session.render_err()),
        }
    }
}
