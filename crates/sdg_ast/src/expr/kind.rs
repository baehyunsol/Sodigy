use super::{Expr, InfixOp, PostfixOp, PrefixOp};
use crate::ast::NameOrigin;
use crate::session::LocalParseSession;
use crate::value::ValueKind;

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
}

impl ExprKind {
    pub fn is_branch(&self) -> bool {
        if let ExprKind::Branch(_, _, _) = self {
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

    pub fn to_string(&self, session: &LocalParseSession) -> String {
        match self {
            ExprKind::Value(v) => v.to_string(session),
            ExprKind::Prefix(op, expr) => format!("{op:?}({})", expr.to_string(session)),
            ExprKind::Infix(op, lhs, rhs) => format!(
                "{}({},{})",
                op.to_string(session),
                lhs.to_string(session),
                rhs.to_string(session),
            ),
            ExprKind::Postfix(op, expr) => format!("{op:?}({})", expr.to_string(session)),
            ExprKind::Call(functor, args) => format!(
                "Call({}{})",
                functor.to_string(session),
                args.iter()
                    .map(|arg| format!(",{}", arg.to_string(session)))
                    .collect::<Vec<String>>()
                    .concat()
            ),
            ExprKind::Branch(cond, t, f) => format!(
                "Branch({},{},{})",
                cond.to_string(session),
                t.to_string(session),
                f.to_string(session)
            ),
        }
    }
}
