use super::{Expr, ExprKind, MirFunc};
use std::fmt;

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

            // TODO: any better way?
            ExprKind::Object(uid) => format!("obj_{:032x}", uid.get_u128()),
            ExprKind::Call { 
                func, args, ..
            } => {
                let args = args.iter().map(
                    |arg| arg.to_string()
                ).collect::<Vec<_>>().join(", ");

                match func {
                    MirFunc::Static(uid) => format!("obj_{:032x}({args})", uid.get_u128()),
                    MirFunc::Dynamic(f) => format!("({f})({args})"),
                }
            },
        };

        write!(fmt, "{s}")
    }
}
