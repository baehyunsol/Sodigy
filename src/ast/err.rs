use crate::ast::NameScope;
use crate::err::SodigyError;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;

mod kind;

pub use kind::ASTErrorKind;

pub struct ASTError {
    kind: ASTErrorKind,
    span1: Span,
    span2: Span,  // optional
}

impl ASTError {
    pub(crate) fn multi_def(name: InternedString, first_def: Span, second_def: Span) -> Self {
        ASTError {
            kind: ASTErrorKind::MultipleDef(name),
            span1: first_def,
            span2: second_def,
        }
    }

    pub(crate) fn deco(span: Span) -> Self {
        ASTError {
            kind: ASTErrorKind::DecoratorOnUse,
            span1: span,
            span2: Span::dummy(),
        }
    }

    pub(crate) fn no_def(name: InternedString, span: Span, name_scope: NameScope) -> Self {
        ASTError {
            kind: ASTErrorKind::UndefinedSymbol(name, name_scope),
            span1: span,
            span2: Span::dummy(),
        }
    }
}

impl SodigyError for ASTError {
    fn render_err(&self, session: &LocalParseSession) -> String {
        self.kind.render_err(self.span1, self.span2, session)
    }
}