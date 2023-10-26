use super::{FormattedStringElement, TokenTree, TokenTreeKind};
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
