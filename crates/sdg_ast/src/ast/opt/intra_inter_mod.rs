use super::super::{AST, ASTError, NameOrigin};
use crate::expr::{Expr, ExprKind, InfixOp, MatchBranch};
use crate::iter_mut_exprs_in_ast;
use crate::path::Path;
use crate::session::{InternedString, LocalParseSession};
use crate::stmt::{ArgDef, Decorator};
use crate::value::{BlockDef, ValueKind};
use sdg_uid::UID;
use std::collections::HashMap;

// It does what `sdg_inter_mod` does, but for locals and preludes. It lessens the burden of compiler because
// `sdg_ast` runs in parallel and takes the advantage of incremental compilation.

pub struct LocalUIDs {
    locals: HashMap<InternedString, UID>,
    preludes: HashMap<InternedString, UID>,
    paths: HashMap<Vec<InternedString>, UID>,
}

impl LocalUIDs {
    pub fn get_uid_from_name(&self, name: InternedString, origin: &NameOrigin) -> Option<UID> {
        match origin {
            NameOrigin::Local => {
                self.locals.get(&name).map(|id| *id)
            },
            NameOrigin::Prelude => {
                self.preludes.get(&name).map(|id| *id)
            },
            _ => None,
        }
    }

    // TODO
    // let's say `p` is `aa.bb.cc`
    // 1. if it has uid for `aa.bb.cc`, it returns `Some(Object(X))`
    // 2. if it has uid for `aa.bb`, it returns `Some(Object(X).cc)`
    // 3. if it has uid for `aa`, it returns `Some(Object(X).bb.cc)`
    // 4. otherwise it returns `None`
    // when converted, it has to preserve the spans
    pub fn try_subst_uid_in_path(&self, p: &Path) -> Option<Expr> {
        let names: Vec<InternedString> = p.as_ref().iter().map(
            |(name, _)| *name
        ).collect();

        for i in 0..(names.len() - 1) {
            let i = names.len() - i;

            if let Some(id) = self.paths.get(&names[0..i]) {
                todo!();
            }
        }

        None
    }
}

impl AST {
    pub(crate) fn get_local_uids(&self) -> LocalUIDs {
        let locals = self.defs.iter().map(
            |(name, def)| (*name, def.id)
        ).collect();

        // TODO: init these
        let preludes = HashMap::new();
        let paths = HashMap::new();

        LocalUIDs {
            locals,
            preludes,
            paths,
        }
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
                        let _ = elem.intra_inter_mod(session, ctxt);
                    }
                },
                ValueKind::Identifier(name, origin) => {
                    if let Some(uid) = ctxt.get_uid_from_name(*name, origin) {
                        self.kind = ExprKind::Value(ValueKind::Object(uid));
                    }
                },
                ValueKind::Closure(_, captured_vars) => {
                    for var in captured_vars.iter_mut() {
                        let _ = var.intra_inter_mod(session, ctxt);
                    }
                },
                // `name_resolve` should remove all the `ValueKind::Lambda`
                ValueKind::Lambda(args, expr) => unreachable!(
                    "Internal Compiler Error B8767A867D2"
                ),
                ValueKind::Block { defs, value, .. } => {
                    let _ = value.intra_inter_mod(session, ctxt);

                    for BlockDef { value, ty, .. } in defs.iter_mut() {
                        let _ = value.intra_inter_mod(session, ctxt);

                        if let Some(ty) = ty {
                            let _ = ty.intra_inter_mod(session, ctxt);
                        }
                    }
                },
            },
            ExprKind::Prefix(_, operand)
            | ExprKind::Postfix(_, operand) => {
                let _ = operand.intra_inter_mod(session, ctxt);
            },
            ExprKind::Infix(op, op1, op2) => match op {
                InfixOp::Path => {
                    if let Some(p) = Path::try_from_expr(self) {
                        if let Some(e) = ctxt.try_subst_uid_in_path(&p) {
                            *self = e;
                        }
                    }
                },
                _ => {
                    let _ = op1.intra_inter_mod(session, ctxt);
                    let _ = op2.intra_inter_mod(session, ctxt);
                }
            },
            ExprKind::Branch(c, t, f) => {
                let _ = c.intra_inter_mod(session, ctxt);
                let _ = t.intra_inter_mod(session, ctxt);
                let _ = f.intra_inter_mod(session, ctxt);
            },
            ExprKind::Call(f, args) => {
                let _ = f.intra_inter_mod(session, ctxt);

                for arg in args.iter_mut() {
                    let _ = arg.intra_inter_mod(session, ctxt);
                }
            },
            ExprKind::Match(value, branches, _) => {
                let _ = value.intra_inter_mod(session, ctxt);

                for MatchBranch { pattern, value, .. } in branches.iter_mut() {
                    let _ = pattern.intra_inter_mod(session, ctxt);
                    let _ = value.intra_inter_mod(session, ctxt);
                }
            },
        }

        // it never fails
        Ok(())
    }
}
