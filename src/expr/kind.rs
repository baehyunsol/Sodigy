use super::{Expr, InfixOp, PostfixOp, PrefixOp};
use crate::ast::NameOrigin;
use crate::token::TokenKind;
use crate::value::ValueKind;

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
}

impl ExprKind {
    pub fn is_identifier(&self) -> bool {
        if let ExprKind::Value(v) = self {
            v.is_identifier()
        } else {
            false
        }
    }

    pub fn is_branch(&self) -> bool {
        if let ExprKind::Branch(_, _, _) = self {
            true
        } else {
            false
        }
    }

    pub fn get_first_token(&self) -> TokenKind {
        match self {
            ExprKind::Value(v) => v.get_first_token(),
            ExprKind::Call(f, _) => f.get_first_token(),
            ExprKind::Infix(_, e, _) | ExprKind::Postfix(_, e) => e.get_first_token(),
            ExprKind::Prefix(op, _) => TokenKind::Operator(op.into()),
            ExprKind::Branch(_, _, _) => TokenKind::keyword_if(),
        }
    }

    pub fn set_origin(&mut self, origin: NameOrigin) {
        match self {
            ExprKind::Value(ValueKind::Identifier(_, curr_origin)) => {
                *curr_origin = origin;
            }
            _ => panic!("Internal Compiler Error 3FDF4E0"),
        }
    }
}
