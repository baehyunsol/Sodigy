use crate::ast::NameScope;
use crate::err::SodigyError;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;

mod kind;

pub use kind::ASTErrorKind;

pub struct ASTError {
    kind: ASTErrorKind,
    span: Vec<Span>,
    message: String,
}

impl ASTError {
    pub(crate) fn multi_def(name: InternedString, first_def: Span, second_def: Span) -> Self {
        ASTError {
            kind: ASTErrorKind::MultipleDef(name),
            span: vec![first_def, second_def],
            message: String::new(),
        }
    }

    pub(crate) fn deco_use(span: Span) -> Self {
        ASTError {
            kind: ASTErrorKind::InvalidDecorator,
            span: vec![span],
            message: String::from("Decorators can only decorate `def` statements,\nbut it's decorating a `use` statement here."),
        }
    }

    pub(crate) fn deco_mod(span: Span) -> Self {
        ASTError {
            kind: ASTErrorKind::InvalidDecorator,
            span: vec![span],
            message: String::from("Decorators can only decorate `def` statements,\nbut it's decorating a `module` statement here."),
        }
    }

    pub(crate) fn no_def(name: InternedString, span: Span, name_scope: NameScope) -> Self {
        ASTError {
            kind: ASTErrorKind::UndefinedSymbol(name, name_scope),
            span: vec![span],
            message: String::new(),
        }
    }

    pub(crate) fn recursive_def(data: Vec<(InternedString, Span)>) -> Self {
        ASTError {
            kind: ASTErrorKind::RecursiveDefInBlock(data.iter().map(|(name, _)| *name).collect()),
            span: data.iter().map(|(_, span)| *span).collect(),
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
            if self.span.is_empty() {
                ""
            } else {
                "\n"
            },
            self.span.iter().map(
                |span| format!("{}", span.render_err(session))
            ).collect::<Vec<String>>().join("\n\n"),
        )
    }

    fn try_add_more_helpful_message(&mut self) {
        if !self.message.is_empty() {
            return;
        }

        match self.kind {
            _ => {}
        }
    }

    fn get_first_span(&self) -> Span {
        if self.span.is_empty() {
            Span::dummy()
        } else {
            let mut curr = self.span[0];

            for span in self.span.iter() {
                if *span < curr {
                    curr = *span;
                }
            }

            curr
        }
    }
}