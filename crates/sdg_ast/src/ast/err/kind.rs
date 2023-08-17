use crate::ast::NameScope;
use crate::session::{InternedString, LocalParseSession};
use crate::utils::{bytes_to_string, print_list};

pub enum ASTErrorKind {
    MultipleDef(InternedString),

    // NameScope is used to suggest similar names
    UndefinedSymbol(InternedString, NameScope),
    InvalidDecorator,

    RecursiveDefInBlock(Vec<InternedString>),
}

impl ASTErrorKind {
    pub fn render_err(&self, session: &LocalParseSession) -> String {
        match self {
            ASTErrorKind::MultipleDef(d) => {
                let name = d.to_string(session);

                format!(
                    "the name `{name}` is defined more than once",
                )
            },
            ASTErrorKind::UndefinedSymbol(d, names) => {
                let suggestions = names.get_similar_name(*d, session);
                let rendered_suggestions = print_list(
                    &suggestions, "`", "`", "or",
                );
                let suggestions = if suggestions.is_empty() {
                    String::new()
                } else if suggestions.len() > 1 {
                    format!("\nSimilar names exist: {}", rendered_suggestions)
                } else {
                    format!("\nA similar name exists: {}", rendered_suggestions)
                };

                let name = session.unintern_string(*d);
                let name = bytes_to_string(&name);

                format!(
                    "cannot find name `{name}` in this scope{suggestions}",
                )
            },
            ASTErrorKind::InvalidDecorator => format!(
                "invalid decorator",
            ),
            ASTErrorKind::RecursiveDefInBlock(names) => {
                let (a, s) = if names.len() == 1 {
                    ("a ", "")
                } else {
                    ("", "s")
                };

                format!(
                    "{a}recursively defined value{s} in a block expression: {}",
                    print_list(
                        &names.iter().map(
                            |name| name.to_string(session)
                        ).collect::<Vec<String>>(),
                        "`", "`", "and"
                    ),
                )
            },
        }
    }
}
