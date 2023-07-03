use crate::expr::ExprKind;
use crate::span::Span;
use crate::token::TokenKind;

mod kind;
mod parse;
#[cfg(test)]
mod tests;

pub use kind::ValueKind;
pub use parse::{parse_block_expr, parse_value};

#[derive(Clone)]
pub struct Value {
    kind: ValueKind,

    // TODO: why do we need span for values when all exprs have a span?
    span: Span,
}

impl Value {
    pub fn is_identifier(&self) -> bool {
        self.kind.is_identifier()
    }

    pub fn get_first_token(&self) -> TokenKind {
        self.kind.get_first_token()
    }

    // `{x = 3; y = 4; x + y}` -> `{x = 3; y = 4; x + y}`
    // `{x + y}` -> `x + y`
    pub fn block_to_expr_kind(&self) -> ExprKind {
        if let ValueKind::Block { defs, value } = &self.kind {
            if defs.is_empty() {
                value.kind.clone()
            } else {
                ExprKind::Value(self.clone())
            }
        } else {
            panic!(
                "Internal Compiler Error 95C0592: {}",
                self.kind.render_err()
            );
        }
    }
}
