use crate::span::Span;
use crate::token::TokenKind;

mod kind;
mod ops;
mod parse;

#[cfg(test)] mod tests;

pub use kind::ExprKind;
pub use ops::{PrefixOp, InfixOp, PostfixOp};
pub use parse::parse_expr;

#[cfg(test)] pub use tests::dump_ast_of_expr;

// `span` points to the first character of the operator
#[derive(Clone)]
pub struct Expr {
    pub span: Span,
    pub kind: ExprKind
}

impl Expr {

    pub fn is_identifier(&self) -> bool {
        self.kind.is_identifier()
    }

    pub fn get_first_token(&self) -> TokenKind {
        self.kind.get_first_token()
    }

}