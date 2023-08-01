use super::{Expr, InfixOp, PostfixOp, PrefixOp};
use crate::ast::NameOrigin;
use crate::pattern::Pattern;
use crate::session::InternedString;
use crate::value::ValueKind;
use sdg_uid::UID;

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

    // value, branches
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
