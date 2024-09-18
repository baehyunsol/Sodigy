use super::{Expr, ExprKind, ValueKind};
use crate::{
    BranchArm,
    FieldKind,
    MatchArm,
    ScopeBlock,
    StructInitDef,
};
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
            ExprKind::Field { pre, post } => format!(
                "({pre}).{post}",
            ),
            ExprKind::Call { func, args } => format!(
                "({func})({})",
                args.iter().map(
                    |arg| arg.to_string()
                ).collect::<Vec<_>>().join(", "),
            ),
            ExprKind::Parenthesis(expr) => format!("({expr})"),
            ExprKind::StructInit {
                struct_, fields,
            } => format!(
                "{struct_} {{{}}}",
                fields.iter().map(
                    |StructInitDef { field, value }| format!("{}: {value}", field.id())
                ).collect::<Vec<_>>().join(", "),
            ),
            ExprKind::Branch(arms) => arms.iter().map(
                |BranchArm {
                    cond,
                    pattern_bind,
                    value,
                    span: _,
                }| format!(
                    "{}{{{value}}}",
                    if let Some(c) = cond {
                        if let Some(p) = pattern_bind {
                            format!("if pattern {p} = {c}")
                        } else {
                            format!("if {c}")
                        }
                    } else {
                        String::new()
                    },
                )
            ).collect::<Vec<_>>().join(" else "),
            ExprKind::Match {
                value,
                arms,
                is_lowered_from_if_pattern: _,
            } => format!(
                "match {value} {{{}}}",
                arms.iter().map(
                    |MatchArm {
                        pattern,
                        guard,
                        value,
                        uid: _,
                    }| format!(
                        "{pattern}{} => {value}",
                        if let Some(g) = guard {
                            format!(" if {g}")
                        } else {
                            String::new()
                        },
                    )
                ).collect::<Vec<_>>().join(", "),
            ),
            ExprKind::Error => String::from("<<COMPILE_ERROR>>"),
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
            ValueKind::Format(elements) => format!(
                "f\"{}\"",
                elements.iter().map(
                    |elem| format!("\\{{{elem}}}")
                ).collect::<Vec<String>>().concat(),
            ),
            ValueKind::Scope { scope: ScopeBlock { lets, value }, .. } => {
                let mut result = Vec::with_capacity(lets.len() + 1);

                for l in lets.iter() {
                    result.push(l.to_string());
                }

                result.push(value.to_string());

                format!("{{{}}}", result.join("; "))
            },
            ValueKind::Lambda {
                args,
                value,
                return_type,
                uid: _,
                lowered_from_scoped_let: _,
            } => {
                let mut result = Vec::with_capacity(args.len() + 1);

                for arg in args.iter() {
                    result.push(arg.to_string());
                }

                result.push(value.to_string());

                format!(
                    "\\{{{}}}{}",
                    result.join(", "),
                    if let Some(ty) = return_type {
                        format!(": {ty}")
                    } else {
                        String::new()
                    },
                )
            },
        };

        write!(fmt, "{s}")
    }
}

impl fmt::Display for FieldKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            FieldKind::Named(n) => n.id().to_string(),
            FieldKind::Index(n) => if *n < 0 {
                format!("_n{n}")
            } else {
                format!("_{n}")
            },
            FieldKind::Range(f, t) => format!("({f}, {t})"),
        };

        write!(fmt, "{s}")
    }
}
