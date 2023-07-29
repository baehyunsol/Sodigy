use crate::session::LocalParseSession;
use crate::span::Span;
use crate::value::ValueKind;

mod kind;
mod ops;
mod parse;
mod name_resolve;

#[cfg(test)]
mod tests;

pub use kind::{ExprKind, MatchBranch};
pub use ops::{InfixOp, PostfixOp, PrefixOp};
pub use parse::{parse_match_body, parse_expr};

#[cfg(test)]
pub use tests::dump_ast_of_expr;

// `span` points to the first character of the operator
#[derive(Clone)]
pub struct Expr {
    pub span: Span,
    pub kind: ExprKind,
}

impl Expr {
    pub fn is_block_with_0_defs(&self) -> bool {
        match &self.kind {
            ExprKind::Value(ValueKind::Block { defs, .. }) if defs.is_empty() => true,
            _ => false,
        }
    }

    pub fn unwrap_block_value(&self) -> Expr {
        match &self.kind {
            ExprKind::Value(ValueKind::Block { value, .. }) => *value.clone(),
            _ => panic!("Internal Compiler Error 0687238F1E8"),
        }
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        match &self.kind {
            ExprKind::Value(v) => v.dump(session, self.span),
            ExprKind::Prefix(op, expr) => format!("{op:?}({})", expr.dump(session)),
            ExprKind::Infix(op, lhs, rhs) => format!(
                "{}({},{})",
                op.dump(session),
                lhs.dump(session),
                rhs.dump(session),
            ),
            ExprKind::Postfix(op, expr) => format!("{op:?}({})", expr.dump(session)),
            ExprKind::Call(functor, args) => format!(
                "Call({}{})",
                functor.dump(session),
                args.iter()
                    .map(|arg| format!(",{}", arg.dump(session)))
                    .collect::<Vec<String>>()
                    .concat()
            ),
            ExprKind::Match(value, branches, _) => format!(
                "Match({},[{}])",
                value.dump(session),
                branches.iter().map(
                    |MatchBranch { pattern, value, .. }| format!(
                        "{}{{{}}}",
                        pattern.dump(session),
                        value.dump(session),
                    )
                ).collect::<Vec<String>>().join(","),
            ),
            ExprKind::Branch(cond, t, f) => {
                #[cfg(test)]
                assert_eq!(self.span.dump(session), "if");

                format!(
                    "Branch({},{},{})",
                    cond.dump(session),
                    t.dump(session),
                    f.dump(session)
                )
            },
        }
    }
}
