use super::NameOrigin;
use crate::expr::{Expr, ExprKind, MatchBranch};
use crate::session::InternedString;
use crate::stmt::ArgDef;
use crate::value::{BlockDef, ValueKind};
use std::collections::HashMap;

mod clean_up_blocks;
mod intra_inter_mod;
mod resolve_recursive_lambdas_in_block;

pub use resolve_recursive_lambdas_in_block::ClosureCollector;
pub use intra_inter_mod::LocalUIDs;

#[derive(Eq, Hash, PartialEq)]
pub enum Opt {
    IntraInterMod,
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

pub fn substitute_local_def(haystack: &mut Expr, substitutions: &HashMap<(InternedString, NameOrigin), Expr>) {
    match &mut haystack.kind {
        ExprKind::Value(v) => match v {
            ValueKind::Identifier(name, origin) => match substitutions.get(&(*name, *origin)) {
                Some(v) => {
                    *haystack = v.clone();
                }
                _ => {}
            },
            ValueKind::Integer(_)
            | ValueKind::Real(_)
            | ValueKind::String(_)
            | ValueKind::Char(_)
            | ValueKind::Bytes(_)
            | ValueKind::Object(_) => {},
            ValueKind::Format(elements)
            | ValueKind::List(elements)
            | ValueKind::Tuple(elements)
            | ValueKind::Closure(_, elements) => {
                for element in elements.iter_mut() {
                    substitute_local_def(element, substitutions);
                }
            },
            ValueKind::Lambda(args, value) => {
                substitute_local_def(value.as_mut(), substitutions);

                for ArgDef { ty, .. } in args.iter_mut() {
                    if let Some(ty) = ty {
                        substitute_local_def(ty, substitutions);
                    }
                }

            },
            ValueKind::Block { defs, value, .. } => {
                substitute_local_def(value.as_mut(), substitutions);

                for BlockDef { value, ty, .. } in defs.iter_mut() {
                    substitute_local_def(value, substitutions);

                    if let Some(ty) = ty {
                        substitute_local_def(ty, substitutions);
                    }
                }
            }
        },
        ExprKind::Prefix(_, op) | ExprKind::Postfix(_, op) => {
            substitute_local_def(op, substitutions);
        },
        ExprKind::Infix(_, op1, op2) => {
            substitute_local_def(op1, substitutions);
            substitute_local_def(op2, substitutions);
        },
        ExprKind::Match(value, branches, _) => {
            substitute_local_def(value, substitutions);

            for MatchBranch { value, .. } in branches.iter_mut() {
                substitute_local_def(value, substitutions);
            }
        }
        ExprKind::Branch(c, t, f) => {
            substitute_local_def(c, substitutions);
            substitute_local_def(t, substitutions);
            substitute_local_def(f, substitutions);
        },
        ExprKind::Call(f, args) => {
            substitute_local_def(f, substitutions);

            for arg in args.iter_mut() {
                substitute_local_def(arg, substitutions);
            }
        },
    }
}
