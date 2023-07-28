use super::{Expr, InfixOp, PostfixOp, PrefixOp};
use crate::ast::NameOrigin;
use crate::pattern::Pattern;
use crate::session::LocalParseSession;
use crate::value::ValueKind;
use sdg_uid::UID;

#[derive(Clone)]
pub enum ExprKind {
    Value(ValueKind),
    Prefix(PrefixOp, Box<Expr>),
    Infix(InfixOp, Box<Expr>, Box<Expr>),
    Postfix(PostfixOp, Box<Expr>),

    // (Functor, Args)
    Call(Box<Expr>, Vec<Expr>),

    // cond, true, false
    Branch(Box<Expr>, Box<Expr>, Box<Expr>),

    // value, branches
    Match(Box<Expr>, Vec<MatchBranch>, UID),
}

impl ExprKind {
    pub fn is_branch(&self) -> bool {
        if let ExprKind::Branch(_, _, _) = self {
            true
        } else {
            false
        }
    }

    pub fn is_match(&self) -> bool {
        if let ExprKind::Match(_, _, _) = self {
            true
        } else {
            false
        }
    }

    pub fn set_origin(&mut self, origin: NameOrigin) {
        match self {
            ExprKind::Value(ValueKind::Identifier(_, curr_origin)) => {
                *curr_origin = origin;
            }
            _ => panic!("Internal Compiler Error 33AC357150A"),
        }
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        match self {
            ExprKind::Value(v) => v.dump(session),
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
            ExprKind::Branch(cond, t, f) => format!(
                "Branch({},{},{})",
                cond.dump(session),
                t.dump(session),
                f.dump(session)
            ),
        }
    }
}

// TODO: where should it belong?
#[derive(Clone)]
pub struct MatchBranch {
    pub(crate) pattern: Pattern,
    pub(crate) value: Expr,
    pub(crate) id: UID,
}
