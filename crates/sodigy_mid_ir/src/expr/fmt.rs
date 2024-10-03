use super::{Expr, ExprKind, MirFunc};
use crate::session::MirSession;
use std::fmt;

impl Expr {
    pub fn render_error(&self, session: &mut MirSession) -> String {
        match &self.kind {
            ExprKind::Integer(n) => n.to_string(),
            ExprKind::LocalValue { key, .. } => format!("_{key}"),
            ExprKind::Object(uid) => session.uid_to_string(*uid),
            ExprKind::Call {
                func, args, ..
            } => {
                let args = args.iter().map(
                    |arg| arg.to_string()
                ).collect::<Vec<_>>().join(", ");

                match func {
                    MirFunc::Static(uid) => format!("{}({args})", session.uid_to_string(*uid)),
                    MirFunc::Dynamic(f) => format!("({f})({args})"),
                }
            },
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.kind.fmt(fmt)
    }
}

impl fmt::Display for ExprKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            ExprKind::Integer(n) => n.to_string(),
            ExprKind::LocalValue { key, .. } => format!("_{key}"),
            ExprKind::Object(uid) => uid.to_ident(),
            ExprKind::Call {
                func, args, ..
            } => {
                let args = args.iter().map(
                    |arg| arg.to_string()
                ).collect::<Vec<_>>().join(", ");

                match func {
                    MirFunc::Static(uid) => format!("{}({args})", uid.to_ident()),
                    MirFunc::Dynamic(f) => format!("({f})({args})"),
                }
            },
        };

        write!(fmt, "{s}")
    }
}
