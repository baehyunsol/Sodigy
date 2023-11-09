use super::{Expr, ExprKind};
use std::fmt;

impl fmt::Display for Expr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.kind)
    }
}

impl fmt::Display for ExprKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            ExprKind::Identifier(id) => format!("{}", id.id()),
            ExprKind::Integer(n)
            | ExprKind::Ratio(n) => format!("{n}"),
            ExprKind::Char(c) => format!("{c:?}"),
            ExprKind::String {
                s, is_binary
            } => format!(
                "{}\"{s}\"",
                if *is_binary { "b" } else { "" },
            ),
            ExprKind::Call { func, args } => format!(
                "{}({})",
                wrap_complicated_exprs(&func.kind),
                args.iter().map(
                    |arg| format!("{arg}")
                ).collect::<Vec<String>>().join(", ")
            ),
            k @ (ExprKind::List(elems)
            | ExprKind::Tuple(elems)) => {
                let is_list = matches!(k, ExprKind::List(_));
                let (start, end) = if matches!(k, ExprKind::List(_)) {
                    ("[", "]")
                } else {
                    ("(", ")")
                };

                format!(
                    "{start}{}{end}",
                    elems.iter().map(
                        |elem| format!("{elem}")
                    ).collect::<Vec<String>>().join(", ")
                )
            },
            ExprKind::Path { head, tail } => format!(
                "{}{}",
                wrap_complicated_exprs(&head.kind),
                tail.iter().map(
                    |id| format!(".{}", id.id())
                ).collect::<Vec<String>>().concat(),
            ),
            _ => String::from("TODO"),
        };

        write!(fmt, "{s}")
    }
}

// like `fmt::Display`, but wraps the result in parenthesis if that makes it more readable
fn wrap_complicated_exprs(e: &ExprKind) -> String {
    match e {
        ExprKind::Identifier(_)
        | ExprKind::Tuple(_) => format!("{e}"),
        _ => format!("({e})"),
    }
}
