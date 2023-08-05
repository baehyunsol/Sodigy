use super::super::{AST, ASTError, NameOrigin};
use crate::expr::{Expr, ExprKind, InfixOp, MatchBranch};
use crate::iter_mut_exprs_in_ast;
use crate::session::{InternedString, LocalParseSession};
use crate::stmt::{ArgDef, Decorator};
use crate::value::{BlockDef, ValueKind};
use sdg_uid::UID;
use std::collections::HashMap;

// It does what `sdg_inter_mod` does, but for locals and preludes. It lessens the burden of compiler because
// `sdg_ast` runs in parallel and takes the advantage of incremental compilation.

// TODO: not only locals, but also preludes
pub type LocalUIDs = HashMap<InternedString, UID>;

impl AST {
    pub(crate) fn get_local_uids(&self) -> LocalUIDs {
        self.defs.iter().map(
            |(name, def)| (*name, def.id)
        ).collect()
    }
}

iter_mut_exprs_in_ast!(intra_inter_mod, LocalUIDs);

impl Expr {
    pub fn intra_inter_mod(
        &mut self,
        session: &LocalParseSession,
        ctxt: &LocalUIDs,
    ) -> Result<(), ASTError> {
        match &mut self.kind {
            ExprKind::Value(v) => match v {
                ValueKind::Integer(_)
                | ValueKind::Real(_)
                | ValueKind::String(_)
                | ValueKind::Char(_)
                | ValueKind::Bytes(_)
                | ValueKind::Object(_) => {},
                ValueKind::List(elements)
                | ValueKind::Tuple(elements)
                | ValueKind::Format(elements) => {
                    for elem in elements.iter_mut() {
                        elem.intra_inter_mod(session, ctxt);
                    }
                },
                ValueKind::Identifier(name, NameOrigin::Local) => {
                    self.kind = ExprKind::Value(ValueKind::Object(
                        *ctxt.get(&name).expect(&format!("Internal Compiler Error 0780D219BE2: {}", name.to_string(session)))
                    ));
                },
                ValueKind::Identifier(_, _) => {},
                ValueKind::Closure(_, captured_vars) => {
                    for var in captured_vars.iter_mut() {
                        var.intra_inter_mod(session, ctxt);
                    }
                },
                // `name_resolve` should remove all the `ValueKind::Lambda`
                ValueKind::Lambda(args, expr) => unreachable!(
                    "Internal Compiler Error B8767A867D2"
                ),
                ValueKind::Block { defs, value, .. } => {
                    value.intra_inter_mod(session, ctxt);

                    for BlockDef { value, ty, .. } in defs.iter_mut() {
                        value.intra_inter_mod(session, ctxt);

                        if let Some(ty) = ty {
                            ty.intra_inter_mod(session, ctxt);
                        }
                    }
                },
            },
            ExprKind::Prefix(_, operand)
            | ExprKind::Postfix(_, operand) => {
                operand.intra_inter_mod(session, ctxt);
            },
            ExprKind::Infix(op, op1, op2) => match op {
                InfixOp::Path => {
                    // let's convert the entire path
                    // `Option.Some(4)` -> `Object(100).Some(4)`
                    // `Option.Some(4)` -> `Object(101)(4)`
                    // both conversions are correct, but the later one is more efficient
                    todo!();
                },
                _ => {
                    op1.intra_inter_mod(session, ctxt);
                    op2.intra_inter_mod(session, ctxt);
                }
            },
            ExprKind::Branch(c, t, f) => {
                c.intra_inter_mod(session, ctxt);
                t.intra_inter_mod(session, ctxt);
                f.intra_inter_mod(session, ctxt);
            },
            ExprKind::Call(f, args) => {
                f.intra_inter_mod(session, ctxt);

                for arg in args.iter_mut() {
                    arg.intra_inter_mod(session, ctxt);
                }
            },
            ExprKind::Match(value, branches, _) => {
                value.intra_inter_mod(session, ctxt);

                for MatchBranch { pattern, value, .. } in branches.iter_mut() {
                    pattern.intra_inter_mod(session, ctxt);
                    value.intra_inter_mod(session, ctxt);
                }
            },
        }

        // it never fails
        Ok(())
    }
}
