use crate::ast::NameScope;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;

pub enum ASTErrorKind {
    MultipleDef(InternedString),

    // NameScope is used to suggest a similar name
    UndefinedSymbol(InternedString, NameScope),
    DecoratorOnUse,
}

impl ASTErrorKind {
    pub fn render_err(&self, span1: Span, span2: Span, session: &LocalParseSession) -> String {
        match self {
            ASTErrorKind::MultipleDef(d) => todo!(),
            ASTErrorKind::UndefinedSymbol(d, names) => {
                let suggestion = names.get_similar_name(*d, session);

                todo!()
            },
            ASTErrorKind::DecoratorOnUse => todo!(),
        }
    }
}