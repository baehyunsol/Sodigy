use super::{Expr, ExprKind, ValueKind};
use crate::ops::InfixOp;
use std::fmt;

impl fmt::Display for Expr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.kind)
    }
}

impl fmt::Display for ExprKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            ExprKind::Value(v) => v.to_string(),
            ExprKind::PrefixOp(op, val) => format!("{op}({val})"),
            ExprKind::PostfixOp(op, val) => format!("({val}){op}"),
            ExprKind::InfixOp(op, lhs, rhs) => if let InfixOp::Index = op {
                format!("({lhs})[{rhs}]")
            } else {
                format!("({lhs}){op}({rhs})")
            },
            ExprKind::Path { pre, post } => format!("({pre}).{}", post.id()),
            ExprKind::Call { func, args } => format!(
                "({func})({})",
                args.iter().map(
                    |arg| arg.to_string()
                ).collect::<Vec<_>>().join(", "),
            ),
            ExprKind::Parenthesis(expr) => format!("({expr})"),
            ExprKind::Error => String::from("<<COMPILE_ERROR>>"),
            _ => todo!(),
        };

        write!(fmt, "{s}")
    }
}

impl fmt::Display for ValueKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            ValueKind::Identifier(id) => id.to_string(),
            ValueKind::Number(n) => n.to_string(),
            ValueKind::String { content, is_binary } => format!(
                "{}\"{}\"",
                if *is_binary { "b" } else { "" },
                content.escaped_no_quotes(),
            ),
            ValueKind::Char(c) => format!("{c:?}"),
            v @ (ValueKind::List(elems)
            | ValueKind::Tuple(elems)) => {
                let is_tuple = matches!(v, ValueKind::Tuple(_));
                let (start, end) = if is_tuple { ("(", ")") } else { ("[", "]") };

                format!(
                    "{start}{}{end}",
                    elems.iter().map(
                        |elem| elem.to_string()
                    ).collect::<Vec<String>>().join(", "),
                )
            },
            _ => todo!(),
        };

        write!(fmt, "{s}")
    }
}
