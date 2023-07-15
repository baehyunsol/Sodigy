use crate::span::Span;
use crate::token::TokenKind;

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
}
