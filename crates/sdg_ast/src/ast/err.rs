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
    message: String,
}

impl ASTError {
    pub(crate) fn multi_def(name: InternedString, first_def: Span, second_def: Span) -> Self {
        ASTError {
            kind: ASTErrorKind::MultipleDef(name),
            span1: first_def,
            span2: second_def,
            message: String::new(),
        }
    }

    pub(crate) fn deco_use(span: Span) -> Self {
        ASTError {
            kind: ASTErrorKind::InvalidDecorator,
            span1: span,
            span2: Span::dummy(),
            message: String::from("Decorators can only decorate `def` statements,\nbut it's decorating a `use` statement here."),
        }
    }

    pub(crate) fn deco_mod(span: Span) -> Self {
        ASTError {
            kind: ASTErrorKind::InvalidDecorator,
            span1: span,
            span2: Span::dummy(),
            message: String::from("Decorators can only decorate `def` statements,\nbut it's decorating a `module` statement here."),
        }
    }

    pub(crate) fn no_def(name: InternedString, span: Span, name_scope: NameScope) -> Self {
        ASTError {
            kind: ASTErrorKind::UndefinedSymbol(name, name_scope),
            span1: span,
            span2: Span::dummy(),
            message: String::new(),
        }
    }

    pub(crate) fn recursive_def(name: InternedString, span: Span) -> Self {
        ASTError {
            kind: ASTErrorKind::RecursiveDefInBlock(name),
            span1: span,
            span2: Span::dummy(),
            message: String::new(),
        }
    }
}

impl SodigyError for ASTError {
    fn render_err(&self, session: &LocalParseSession) -> String {
        format!(
            "Error: {}{}{}{}",
            self.kind.render_err(session),
            if self.message.is_empty() {
                String::new()
            } else {
                format!("\n{}", self.message)
            },
            if self.span1.is_dummy() {
                String::new()
            } else {
                format!(
                    "\n{}",
                    self.span1.render_err(session),
                )
            },
            if self.span2.is_dummy() {
                String::new()
            } else {
                format!(
                    "\n{}",
                    self.span2.render_err(session),
                )
            },
        )
    }
}