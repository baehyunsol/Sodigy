use super::{
    Branch,
    BranchArm,
    Expr,
    ExprKind,
    Lambda,
    Match,
    MatchArm,
    Scope,
    ScopedLet,
    StructInit,
    StructInitField,
};
use crate::func::Arg;
use sodigy_ast::InfixOp;
use std::fmt;

impl fmt::Display for Expr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.kind)
    }
}

impl fmt::Display for ExprKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            ExprKind::Identifier(id) => id.id().to_string(),
            ExprKind::Integer(n)
            | ExprKind::Ratio(n) => n.to_string(),
            ExprKind::Char(c) => format!("{c:?}"),
            ExprKind::String {
                content, is_binary
            } => format!(
                "{}\"{}\"",
                if *is_binary { "b" } else { "" },
                content.escaped_no_quotes(),
            ),
            ExprKind::Call { func, args } => format!(
                "{}({})",
                wrap_complicated_exprs(&func.kind),
                args.iter().map(
                    |arg| arg.to_string()
                ).collect::<Vec<String>>().join(", ")
            ),
            k @ (ExprKind::List(elems)
            | ExprKind::Tuple(elems)) => {
                let (start, end) = if matches!(k, ExprKind::List(_)) {
                    ("[", "]")
                } else {
                    ("(", ")")
                };

                format!(
                    "{start}{}{end}",
                    elems.iter().map(
                        |elem| elem.to_string()
                    ).collect::<Vec<String>>().join(", ")
                )
            },
            ExprKind::Format(elems) => format!(
                "f\"{}\"",
                elems.iter().map(
                    |elem| match &elem.kind {
                        ExprKind::String { content, is_binary } => {
                            debug_assert!(!*is_binary);
                            content.escaped_no_quotes()
                        },
                        _ => format!("\\{{{elem}}}"),
                    }
                ).collect::<Vec<String>>().concat(),
            ),
            ExprKind::Scope(Scope { lets, value, .. }) => {
                let mut result = Vec::with_capacity(lets.len() + 1);

                for ScopedLet { name, value, ty, .. } in lets.iter() {
                    result.push(format!(
                        "let {}{} = {value}",
                        name.id(),
                        if let Some(ty) = ty {
                            format!(": {ty}")
                        } else {
                            String::new()
                        },
                    ));
                }

                result.push(value.to_string());

                format!("{{{}}}", result.join("; "))
            },
            ExprKind::Match(Match { arms, value, .. }) => {
                let mut arms_rendered = Vec::with_capacity(arms.len());

                for MatchArm { pattern, value, guard } in arms.iter() {
                    arms_rendered.push(format!(
                        "{pattern}{} => {value}",
                        if let Some(guard) = guard {
                            format!(" if {guard}")
                        } else {
                            String::new()
                        },
                    ));
                }

                let arms_rendered = arms_rendered.join(", ");

                format!("match {value} {{{arms_rendered}}}")
            },
            ExprKind::Lambda(Lambda { args, value, .. }) => {
                let mut result = Vec::with_capacity(args.len() + 1);

                for Arg { name, ty, has_question_mark, attributes } in args.iter() {
                    // TODO: render `attributes`
                    result.push(format!(
                        "{}{}{}",
                        name.id(),
                        if *has_question_mark { "?" } else { "" },
                        if let Some(ty) = ty {
                            format!(": {ty}")
                        } else {
                            String::new()
                        },
                    ));
                }

                result.push(value.to_string());

                format!("\\{{{}}}", result.join(", "))
            },
            ExprKind::Branch(Branch { arms }) => {
                let mut result = Vec::with_capacity(arms.len());

                for (index, BranchArm { cond, value }) in arms.iter().enumerate() {
                    result.push(format!(
                        "{}{}{}",
                        if index == 0 {
                            "if "
                        } else if cond.is_some() {
                            "else if "
                        } else {
                            "else "
                        },
                        if let Some(cond) = cond {
                            format!("{cond} ")
                        } else {
                            String::new()
                        },
                        {
                            // pretty print: remove unnecessary braces
                            let v = value.to_string();

                            if v.starts_with("{") {
                                v
                            }

                            else {
                                format!("{{{v}}}")
                            }
                        }
                    ));
                }

                result.join(" ")
            },
            ExprKind::StructInit(StructInit { struct_, fields }) => {
                let mut fields_rendered = Vec::with_capacity(fields.len());

                for StructInitField { name, value } in fields.iter() {
                    fields_rendered.push(format!("{}: {value}", name.id()));
                }

                format!(
                    "{} {{{}}}",
                    wrap_unless_name(&struct_.kind),
                    fields_rendered.join(", ")
                )
            },
            ExprKind::Field { pre, post } => format!(
                "{}.{post}",
                wrap_complicated_exprs(&pre.kind),
            ),
            ExprKind::PrefixOp(op, val) => format!("{op}{}", wrap_if_op(&val.kind)),
            ExprKind::PostfixOp(op, val) => format!("{}{op}", wrap_if_op(&val.kind)),
            ExprKind::InfixOp(op, lhs, rhs) => if let InfixOp::Index = op {
                format!("{}[{rhs}]", wrap_if_op(&lhs.kind))
            } else {
                format!("{}{op}{}", wrap_if_op(&lhs.kind), wrap_if_op(&rhs.kind))
            },
        };

        write!(fmt, "{s}")
    }
}

// like `fmt::Display`, but wraps the result in parenthesis if that makes it more readable
fn wrap_complicated_exprs(e: &ExprKind) -> String {
    match e {
        ExprKind::Identifier(_)
        | ExprKind::Tuple(_) => e.to_string(),
        _ => format!("({e})"),
    }
}

// when multiple operators are nested, it's safe to wrap them in parenthesis
// otherwise precedences may mess up stuffs
fn wrap_if_op(e: &ExprKind) -> String {
    match e {
        ExprKind::InfixOp(..)
        | ExprKind::PrefixOp(..)
        | ExprKind::PostfixOp(..) => format!("({e})"),
        _ => e.to_string(),
    }
}

fn wrap_unless_name(e: &ExprKind) -> String {
    match e {
        ExprKind::Identifier(_)
        | ExprKind::Field { .. } => e.to_string(),
        _ => format!("({e})"),
    }
}
