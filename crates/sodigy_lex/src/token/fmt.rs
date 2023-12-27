use super::{CommentKind, Token, TokenKind};
use std::fmt;

impl fmt::Debug for TokenKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            match self {
                TokenKind::Comment { kind, content } => format!("Comment({kind:?}, {content:?})"),
                TokenKind::String { kind, content, .. } => format!("String({kind:?}, {content:?})"),
                TokenKind::Identifier(id) => format!("Identifier({id:?})"),
                TokenKind::Number(n) => format!("Number({:?})", String::from_utf8(n.to_vec()).unwrap()),
                TokenKind::Whitespace => format!("Whitespace"),
                TokenKind::Punct(p) => format!("Punct({:?})", *p as char),
                TokenKind::Grouper(g) => format!("Grouper({:?})", *g as char),
            }
        )
    }
}

impl fmt::Display for Token {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.kind)
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            match self {
                TokenKind::Comment { kind, content } => match kind {
                    CommentKind::Single => format!("# ...\n"),
                    CommentKind::Multi => format!("##! ... !##"),
                    CommentKind::Doc => format!("##>{}\n", content),
                },
                TokenKind::String { kind, content, .. } => {
                    let result = format!("{content:?}").as_bytes().to_vec();
                    let result = result[1..(result.len() - 1)].to_vec();

                    format!(
                        "{}{}{}",
                        *kind as u8 as char,
                        String::from_utf8_lossy(&result).to_string(),
                        *kind as u8 as char,
                    )
                },
                TokenKind::Identifier(id) => id.to_string(),
                TokenKind::Number(n) => String::from_utf8(n.to_vec()).unwrap(),
                TokenKind::Whitespace => format!(" "),
                TokenKind::Punct(p) => format!("{}", *p as char),
                TokenKind::Grouper(g) => format!("{}", *g as char),
            }
        )
    }
}
