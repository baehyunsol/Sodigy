use super::{
    Branch,
    BranchArm,
    Expr,
    ExprKind,
    Lambda,
    LocalDef,
    Match,
    MatchArm,
    Scope,
    StructInit,
    StructInitField,
};
use crate::func::Arg;
use sodigy_ast::InfixOp;
use sodigy_test::sodigy_assert;
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
            ExprKind::Format(elems) => format!(
                "f\"{}\"",
                elems.iter().map(
                    |elem| match &elem.kind {
                        ExprKind::String { s, is_binary } => {
                            sodigy_assert!(!*is_binary);
                            format!("{s}")
                        },
                        _ => format!("{{{elem}}}"),
                    }
                ).collect::<Vec<String>>().concat(),
            ),
            ExprKind::Scope(Scope { defs, value, .. }) => {
                let mut result = Vec::with_capacity(defs.len() + 1);

                for LocalDef { pattern, value, .. } in defs.iter() {
                    result.push(format!("let {pattern} = {value}"));
                }

                result.push(format!("{value}"));

                format!("{{{}}}", result.join("; "))
            },
            ExprKind::Match(Match { arms, value }) => {
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

                for Arg { name, ty, has_question_mark } in args.iter() {
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

                result.push(format!("{value}"));

                format!("\\{{{}}}", result.join(", "))
            },
            ExprKind::Branch(Branch { arms }) => {
                let mut result = Vec::with_capacity(arms.len());

                for (ind, BranchArm { cond, let_bind, value }) in arms.iter().enumerate() {
                    result.push(format!(
                        "{}{}{{{value}}}",
                        if ind == 0 {
                            "if "
                        } else if cond.is_some() {
                            "else if "
                        } else {
                            "else "
                        },
                        if let Some(let_bind) = let_bind {
                            format!(
                                "let {} = {}",
                                let_bind,
                                cond.as_ref().unwrap(),
                            )
                        } else {
                            if let Some(cond) = cond {
                                format!("{cond} ")
                            } else {
                                String::new()
                            }
                        },
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
            ExprKind::Path { head, tail } => format!(
                "{}{}",
                wrap_complicated_exprs(&head.kind),
                tail.iter().map(
                    |id| format!(".{}", id.id())
                ).collect::<Vec<String>>().concat(),
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
        | ExprKind::Tuple(_) => format!("{e}"),
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
        _ => format!("{e}"),
    }
}

fn wrap_unless_name(e: &ExprKind) -> String {
    match e {
        ExprKind::Identifier(_)
        | ExprKind::Path { .. } => format!("{e}"),
        _ => format!("({e})"),
    }
}
