use crate::ast::NameScope;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::utils::bytes_to_string;

pub enum ASTErrorKind {
    MultipleDef(InternedString),

    // NameScope is used to suggest a similar name
    UndefinedSymbol(InternedString, NameScope),
    DecoratorOnUse,
}

impl ASTErrorKind {
    pub fn render_err(&self, span1: Span, span2: Span, session: &LocalParseSession) -> String {
        match self {
            ASTErrorKind::MultipleDef(d) => {
                let name = session.unintern_string(*d);
                let name = bytes_to_string(&name);

                format!(
                    "`{name}` is defined more than once!\n{}\n\n{}",
                    span1.render_err(session),
                    span2.render_err(session),
                )
            },
            ASTErrorKind::UndefinedSymbol(d, names) => {
                let suggestions = names.get_similar_name(*d, session);
                let suggestions = if suggestions.is_empty() {
                    String::new()
                } else {
                    format!("\nSimilar names found: {}", render_suggestions(suggestions))
                };

                let name = session.unintern_string(*d);
                let name = bytes_to_string(&name);

                format!(
                    "`{name}` is not defined!{suggestions}\n{}",
                    span1.render_err(session),
                )
            },
            ASTErrorKind::DecoratorOnUse => format!(
                "You cannot decorate a `use` statement!\n{}",
                span1.render_err(session),
            ),
        }
    }
}

fn render_suggestions(suggestions: Vec<String>) -> String {
    assert!(!suggestions.is_empty(), "Internal Compiler Error B688677");

    if suggestions.len() == 1 {
        format!("`{}`", suggestions[0])
    }

    else if suggestions.len() == 2 {
        format!("`{}` or `{}`", suggestions[0], suggestions[1])
    }

    else {
        format!("`{}`, {}", suggestions[0], render_suggestions(suggestions[1..].to_vec()))
    }

}