use super::{FormattedStringElement, TokenTree, TokenTreeKind};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;
use std::fmt;

impl fmt::Display for TokenTree {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            self.kind,
        )
    }
}

impl fmt::Display for TokenTreeKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            match self {
                TokenTreeKind::Identifier(id) => format!("{id}"),
                TokenTreeKind::Keyword(keyword) => format!("{keyword}"),
                TokenTreeKind::Number(n) => format!("{n}"),
                TokenTreeKind::Punct(p) => format!("{p}"),
                TokenTreeKind::Group { delim, tokens, prefix } => format!(
                    "{}{}{}{}",
                    if *prefix == b'\0' { String::new() } else { format!("{}", *prefix as char) },
                    delim.start() as char,
                    tokens.iter().map(|t| format!("{t}")).collect::<Vec<String>>().join(" "),
                    delim.end() as char,
                ),
                TokenTreeKind::String { kind, content, is_binary } => format!(
                    "{}{}{}{}",
                    if *is_binary { "b" } else { "" },
                    *kind as u8 as char,
                    content,
                    *kind as u8 as char,
                ),
                TokenTreeKind::FormattedString(elems) => {
                    // TODO: escaped strings
                    format!(
                        "f\"{}\"",
                        elems.iter().map(
                            |elem| match elem {
                                FormattedStringElement::Literal(l) => l.to_string(),
                                FormattedStringElement::Value(v) => format!(
                                    "{{{}}}",
                                    v.iter().map(
                                        |v| format!("{v}")
                                    ).collect::<Vec<String>>().join(" "),
                                ),
                            }
                        ).collect::<Vec<String>>().concat(),
                    )
                },
                TokenTreeKind::DocComment(content) => format!("##>{content}\n"),
            },
        )
    }
}

impl TokenTreeKind {
    /// `fmt::Display` for error messages
    pub fn render_error(&self) -> String {
        match self {
            TokenTreeKind::Identifier(_)
            | TokenTreeKind::Keyword(_)
            | TokenTreeKind::Number(_)
            | TokenTreeKind::Punct(_) => format!("{self}"),
            TokenTreeKind::Group { delim, prefix, .. } => format!(
                "{}",
                TokenTreeKind::Group {
                    delim: *delim,
                    prefix: *prefix,
                    tokens: vec![
                        TokenTree::new_ident(InternedString::dotdotdot(), SpanRange::dummy()),
                    ],
                }
            ),
            TokenTreeKind::String { kind, is_binary, .. } => format!(
                "{}",
                TokenTreeKind::String {
                    kind: *kind,
                    is_binary: *is_binary,
                    content: InternedString::dotdotdot(),
                },
            ),
            TokenTreeKind::FormattedString(_) => String::from("f\"...\""),
            TokenTreeKind::DocComment(_) => String::from("##> ..."),
        }
    }
}
