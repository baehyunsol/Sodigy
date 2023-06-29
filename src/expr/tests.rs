use super::{Expr, ExprKind};
use crate::session::LocalParseSession;

mod dump_ast;

pub use dump_ast::dump_ast_of_expr;

impl Expr {

    pub fn to_string(&self, session: &LocalParseSession) -> String {
        self.kind.to_string(session)
    }

}

impl ExprKind {

    pub fn to_string(&self, session: &LocalParseSession) -> String {

        match self {
            ExprKind::Value(v) => v.to_string(session),
            ExprKind::Prefix(op, expr) => format!("{op:?}({})", expr.to_string(session)),
            ExprKind::Infix(op, lhs, rhs) => format!("{op:?}({},{})", lhs.to_string(session), rhs.to_string(session)),
            ExprKind::Postfix(op, expr) => format!("{op:?}({})", expr.to_string(session)),
            ExprKind::Call(functor, args) => format!(
                "Call({}{})",
                functor.to_string(session),
                args.iter().map(
                    |arg| format!(",{}", arg.to_string(session))
                ).collect::<Vec<String>>().concat()
            ),
            ExprKind::Branch(cond, t, f) => format!(
                "Branch({},{},{})",
                cond.to_string(session),
                t.to_string(session),
                f.to_string(session)
            )
        }

    }

}