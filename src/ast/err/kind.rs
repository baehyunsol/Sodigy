use crate::ast::NameScope;
use crate::session::{InternedString, LocalParseSession};
use crate::utils::bytes_to_string;

pub enum ASTErrorKind {
    MultipleDef(InternedString),

    // NameScope is used to suggest a similar name
    UndefinedSymbol(InternedString, NameScope),
    DecoratorOnUse,

    RecursiveDefInBlock(InternedString),
}

impl ASTErrorKind {
    pub fn render_err(&self, session: &LocalParseSession) -> String {
        match self {
            ASTErrorKind::MultipleDef(d) => {
                let name = session.unintern_string(*d);
                let name = bytes_to_string(&name);

                format!(
                    "`{name}` is defined more than once!",
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
                    "`{name}` is not defined!{suggestions}",
                )
            },
            ASTErrorKind::DecoratorOnUse => format!(
                "You cannot decorate a `use` statement!",
            ),
            ASTErrorKind::RecursiveDefInBlock(name) => format!(
                "A block expression contains a recursively defined value: `{}`",
                bytes_to_string(&session.unintern_string(*name)),
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