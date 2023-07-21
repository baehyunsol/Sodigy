use crate::span::Span;
use crate::token::TokenKind;
use crate::value::ValueKind;

mod kind;
mod ops;
mod parse;
mod name_resolve;

#[cfg(test)]
mod tests;

pub use kind::ExprKind;
pub use ops::{InfixOp, PostfixOp, PrefixOp};
pub use parse::parse_expr;

#[cfg(test)]
pub use tests::dump_ast_of_expr;

// `span` points to the first character of the operator
#[derive(Clone)]
pub struct Expr {
    pub span: Span,
    pub kind: ExprKind,
}

impl Expr {
    pub fn get_first_token(&self) -> TokenKind {
        self.kind.get_first_token()
    }

    pub fn is_block_with_0_defs(&self) -> bool {
        match &self.kind {
            ExprKind::Value(ValueKind::Block { defs, .. }) if defs.is_empty() => true,
            _ => false,
        }
    }

    pub fn unwrap_block_value(&self) -> Expr {
        match &self.kind {
            ExprKind::Value(ValueKind::Block { value, .. }) => *value.clone(),
            _ => panic!("Internal Compiler Error B28E693"),
        }
    }
}
