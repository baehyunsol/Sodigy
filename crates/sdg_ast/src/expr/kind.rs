use super::{Expr, InfixOp, PostfixOp, PrefixOp};
use crate::ast::NameOrigin;
use crate::pattern::Pattern;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::value::ValueKind;
use sdg_uid::UID;

#[derive(Clone)]
pub enum ExprKind {
    Value(ValueKind),
    Prefix(PrefixOp, Box<Expr>),
    Infix(InfixOp, Box<Expr>, Box<Expr>),
    Postfix(PostfixOp, Box<Expr>),

    /// (Functor, Args)
    Call(Box<Expr>, Vec<Expr>),

    /// cond, true, false
    Branch(Box<Expr>, Box<Expr>, Box<Expr>),

    /// value, branches
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

    pub fn is_closure(&self) -> bool {
        if let ExprKind::Value(ValueKind::Closure(_, _)) = self {
            true
        } else {
            false
        }
    }

    pub fn unwrap_closure_name(&self) -> InternedString {
        if let ExprKind::Value(ValueKind::Closure(name, _)) = self {
            *name
        } else {
            unreachable!("Internal Compiler Error F77F5A131D0")
        }
    }

    pub fn is_lambda(&self) -> bool {
        if let ExprKind::Value(ValueKind::Lambda(_, _)) = self {
            true
        } else if let ExprKind::Value(ValueKind::Identifier(_, origin)) = self {
            *origin == NameOrigin::AnonymousFunc
        } else {
            false
        }
    }

    pub fn unwrap_lambda_name(&self) -> InternedString {
        if let ExprKind::Value(ValueKind::Identifier(name, _)) = self {
            *name
        } else {
            unreachable!("Internal Compiler Error C8FB679F0CB")
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

    pub fn dump(&self, session: &LocalParseSession, span: Span) -> String {
        match self {
            ExprKind::Value(v) => v.dump(session, span),
            ExprKind::Prefix(op, expr) => format!("{op:?}({})", expr.dump(session)),
            ExprKind::Infix(op, lhs, rhs) => format!(
                "{}({}, {})",
                op.dump(session),
                lhs.dump(session),
                rhs.dump(session),
            ),
            ExprKind::Postfix(op, expr) => format!("{op:?}({})", expr.dump(session)),
            ExprKind::Call(functor, args) => format!(
                "Call({}{})",
                functor.dump(session),
                args.iter()
                    .map(|arg| format!(", {}", arg.dump(session)))
                    .collect::<Vec<String>>()
                    .concat()
            ),
            ExprKind::Match(value, branches, _) => format!(
                "Match({}, [{}])",
                value.dump(session),
                branches.iter().map(
                    |MatchBranch { pattern, value, .. }| format!(
                        "{} => {}",
                        pattern.dump(session),
                        value.dump(session),
                    )
                ).collect::<Vec<String>>().join(", "),
            ),
            ExprKind::Branch(cond, t, f) => {
                #[cfg(test)]
                assert_eq!(span.dump(session), "if");

                format!(
                    "Branch({}, {}, {})",
                    cond.dump(session),
                    t.dump(session),
                    f.dump(session)
                )
            },
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

impl MatchBranch {
    pub fn new(pattern: Pattern, value: Expr) -> Self {
        MatchBranch {
            pattern, value,
            id: UID::new_match_branch_id(),
        }
    }
}
