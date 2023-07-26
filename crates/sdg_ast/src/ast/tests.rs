use super::AST;
use crate::session::LocalParseSession;
use crate::parse_file;

mod block_clean_up;
mod name_origin;
mod name_resolve;

impl AST {

    pub fn dump_ast_of_def(&self, def: Vec<u8>, session: &LocalParseSession) -> Option<String> {
        let func_name = if let Some(s) = session.try_intern_string(def) {
            s
        } else {
            return None;
        };

        let func = if let Some(f) = self.defs.get(&func_name) {
            f
        } else {
            return None;
        };

        Some(func.ret_val.dump(session))
    }

}

fn check_ast_of_tester(samples: Vec<(Vec<u8>, String)>) {
    let mut session = LocalParseSession::new();
    let test_func_name = b"tester".to_vec();

    for (sample, desired) in samples.into_iter() {
        session.set_direct_input(sample.clone());
        let ast = match parse_file(&sample, &mut session) {
            Ok(a) => a,
            Err(_) => panic!("{}", session.render_err()),
        };

        assert_eq!(ast.dump_ast_of_def(test_func_name.clone(), &session).unwrap(), desired);
    }

}