use super::AST;
use crate::session::LocalParseSession;

mod clean_up_blocks;
mod resolve_recursive_lambdas_in_block;

pub use resolve_recursive_lambdas_in_block::ClosureCollector;

impl AST {

    pub(crate) fn opt(&mut self, session: &mut LocalParseSession) {
        // TODO
    }

}

#[macro_export]
// make sure that `Expr` implements `$method_name(&mut self, &mut LocalParseSession)`
macro_rules! iter_mut_exprs_in_ast {
    ($method_name: ident, $ctxt: ty) => {
        impl AST {
            pub(crate) fn $method_name(&mut self, session: &mut LocalParseSession, ctxt: &mut $ctxt) -> Result<(), ()> {

                for func in self.defs.values_mut() {
                    for Decorator { args, .. } in func.decorators.iter_mut() {
                        for arg in args.iter_mut() {
                            let e = arg.$method_name(session, ctxt);
                            session.try_add_error(e);
                        }
                    }

                    for ArgDef { ty, .. } in func.args.iter_mut() {
                        if let Some(ty) = ty {
                            let e = ty.$method_name(session, ctxt);
                            session.try_add_error(e);
                        }
                    }

                    if let Some(ty) = &mut func.ret_type {
                        let e = ty.$method_name(session, ctxt);
                        session.try_add_error(e);
                    }

                    let e = func.ret_val.$method_name(session, ctxt);
                    session.try_add_error(e);
                }

                if session.has_no_error() {
                    Ok(())
                }

                else {
                    Err(())
                }

            }
        }
    }
}
