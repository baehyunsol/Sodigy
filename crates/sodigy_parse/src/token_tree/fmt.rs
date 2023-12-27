use super::{FormattedStringElement, TokenTree, TokenTreeKind};
use sodigy_error::RenderError;
use sodigy_intern::try_intern_short_string;
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
                TokenTreeKind::Identifier(id) => id.to_string(),
                TokenTreeKind::Keyword(keyword) => keyword.to_string(),
                TokenTreeKind::Number(n) => n.to_string(),
                TokenTreeKind::Punct(p) => p.to_string(),
                TokenTreeKind::Group { delim, tokens, prefix } => format!(
                    "{}{}{}{}",
                    if *prefix == b'\0' { String::new() } else { format!("{}", *prefix as char) },
                    delim.start() as char,
                    tokens.iter().map(|t| t.to_string()).collect::<Vec<String>>().join(" "),
                    delim.end() as char,
                ),
                TokenTreeKind::String { kind, content, is_binary } => format!(
                    "{}{}{content}{}",
                    if *is_binary { "b" } else { "" },
                    *kind as u8 as char,
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
                                    "\\{{{}}}",
                                    v.iter().map(
                                        |v| v.to_string()
                                    ).collect::<Vec<String>>().join(" "),
                                ),
                            }
                        ).collect::<Vec<String>>().concat(),
                    )
                },
                TokenTreeKind::DocComment(content) => format!("##>{content}\n"),
                TokenTreeKind::Macro { name, args } => format!(
                    "@[{}]({})",
                    name.iter().map(
                        |n| n.to_string()
                    ).collect::<Vec<String>>().join(" "),
                    args.iter().map(
                        |a| a.to_string()
                    ).collect::<Vec<String>>().join(" "),
                ),
            },
        )
    }
}

impl RenderError for TokenTreeKind {
    fn render_error(&self) -> String {
        match self {
            TokenTreeKind::Identifier(_)
            | TokenTreeKind::Keyword(_)
            | TokenTreeKind::Number(_)
            | TokenTreeKind::Punct(_) => self.to_string(),
            TokenTreeKind::Group { delim, prefix, .. } => format!(
                "{}",
                TokenTreeKind::Group {
                    delim: *delim,
                    prefix: *prefix,
                    tokens: vec![
                        TokenTree::new_ident(
                            try_intern_short_string(b"...").unwrap(),
                            SpanRange::dummy(0x368421a5),
                        ),
                    ],
                }
            ),
            TokenTreeKind::String { kind, is_binary, .. } => format!(
                "{}",
                TokenTreeKind::String {
                    kind: *kind,
                    is_binary: *is_binary,
                    content: try_intern_short_string(b"...").unwrap(),
                },
            ),
            TokenTreeKind::FormattedString(_) => String::from("f\"...\""),
            TokenTreeKind::DocComment(_) => String::from("##> ..."),
            TokenTreeKind::Macro { name, .. } => format!(
                "@[{}](...)",
                name.iter().map(|n| n.to_string()).collect::<Vec<String>>().join(" "),
            ),
        }
    }
}
