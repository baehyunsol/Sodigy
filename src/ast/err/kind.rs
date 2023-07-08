use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;

pub enum ASTErrorKind {
    MultipleDef(InternedString),
    UndefinedSymbol(InternedString),
    DecoratorOnUse,
}

impl ASTErrorKind {
    pub fn render_err(&self, span1: Span, span2: Span, session: &LocalParseSession) -> String {
        match self {
            ASTErrorKind::MultipleDef(d) => todo!(),
            ASTErrorKind::UndefinedSymbol(d) => todo!(),
            ASTErrorKind::DecoratorOnUse => todo!(),
        }
    }
}