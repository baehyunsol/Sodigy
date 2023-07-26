use super::{Delimiter, Keyword, OpToken, Token};
use crate::session::{InternedString, LocalParseSession};
use hmath::Ratio;

#[derive(Clone, PartialEq)]
pub enum TokenKind {
    Number(Ratio),
    String(Vec<u32>),  // in Sodigy, Strings are just List(Char), where Char is an Int

    // It doesn't care how the inside looks like. It only guarantees that the opening and the closing are properly matched.
    List(Delimiter, Vec<Token>),

    Identifier(InternedString),
    Keyword(Keyword),

    Operator(OpToken),

    // b"ABC" -> [65, 66, 67]
    Bytes(Vec<u8>),

    // f"{a} + {b} = {a + b}" -> a.to_string() <> " + " <> b.to_string() <> " = " <> (a + b).to_string()
    FormattedString(Vec<Vec<Token>>),
}

impl TokenKind {
    pub fn is_identifier(&self) -> bool {
        if let TokenKind::Identifier(_) = self {
            true
        } else {
            false
        }
    }

    pub fn unwrap_identifier(&self) -> InternedString {
        if let TokenKind::Identifier(s) = self {
            *s
        } else {
            panic!(
                "Internal Compiler Error FD9DC4CD703: {}",
                self.render_err(&LocalParseSession::dummy()),
            )
        }
    }
    pub fn is_number(&self) -> bool {
        if let TokenKind::Number(_) = self {
            true
        } else {
            false
        }
    }

    pub fn unwrap_number(&self) -> Ratio {
        if let TokenKind::Number(s) = self {
            s.clone()
        } else {
            panic!(
                "Internal Compiler Error F08E01AE8B0: {}",
                self.render_err(&LocalParseSession::dummy()),
            )
        }
    }

    pub fn is_string(&self) -> bool {
        if let TokenKind::String(_) = self {
            true
        } else {
            false
        }
    }

    pub fn unwrap_string(&self) -> &Vec<u32> {
        if let TokenKind::String(v) = self {
            v
        } else {
            panic!(
                "Internal Compiler Error E58B67B9AFA: {}",
                self.render_err(&LocalParseSession::dummy()),
            )
        }
    }

    pub fn is_stmt_begin(&self) -> bool {
        match self {
            TokenKind::Keyword(k) if *k == Keyword::Use || *k == Keyword::Def => true,
            TokenKind::Operator(OpToken::At) => true,
            _ => false,
        }
    }

    pub fn dummy_identifier() -> Self {
        TokenKind::Identifier(InternedString::dummy())
    }

    // dump vs render_err vs to_string
    // dump is for compiler developers
    // render_err is for compiler users
    // to_string is somewhere in the middle
    pub fn dump(&self, session: &LocalParseSession) -> String {
        match self {
            TokenKind::Number(n) => format!("{n}"),
            TokenKind::String(s) => format!(
                "{:?}",
                s.iter().map(
                    |n| char::from_u32(*n).unwrap()
                ).collect::<String>()
            ),
            TokenKind::Bytes(b) => format!("Bytes({b:?})"),
            TokenKind::FormattedString(s) => format!(
                "Format({})",
                s.iter().map(
                    |s| format!(
                        "[{}]",
                        s.iter().map(
                            |s| s.dump(session)
                        ).collect::<Vec<String>>().join(", ")
                    )
                ).collect::<Vec<String>>().join(", "),
            ),
            TokenKind::List(delim, elements) => format!(
                "{}{}{}",
                delim.start() as char,
                elements.iter().map(
                    |e| e.dump(session)
                ).collect::<Vec<String>>().join(", "),
                delim.end() as char,
            ),
            TokenKind::Identifier(id) => id.to_string(session),
            TokenKind::Operator(op) => op.render_err(),
            TokenKind::Keyword(k) => k.render_err(),
        }
    }

    // preview of this token_kind for error messages
    pub fn render_err(&self, session: &LocalParseSession) -> String {
        match self {
            TokenKind::Number(_) => "a number literal".to_string(),
            TokenKind::String(_) => "a string literal".to_string(),
            TokenKind::Bytes(_) => "a bytes literal".to_string(),
            TokenKind::FormattedString(_) => "a formatted string literal".to_string(),
            TokenKind::List(delim, _) => format!("{}", delim.start() as char),
            TokenKind::Identifier(string) => {
                if string.is_dummy() || session.is_dummy {
                    "an identifier".to_string()
                } else {
                    format!(
                        "an identifier `{}`",
                        string.to_string(session),
                    )
                }
            }
            TokenKind::Keyword(k) => format!("keyword `{}`", k.render_err()),
            TokenKind::Operator(op) => {
                let op = op.render_err();
                let ch = if op.len() == 1 {
                    "character"
                } else {
                    "characters"
                };

                format!("{ch} `{op}`")
            },
        }
    }

    #[cfg(test)]
    pub fn is_same_type(&self, other: &TokenKind) -> bool {
        match (self, other) {
            (TokenKind::Number(m), TokenKind::Number(n)) => m == n,
            (TokenKind::Keyword(m), TokenKind::Keyword(n)) => m == n,
            (TokenKind::Operator(m), TokenKind::Operator(n)) => m == n,

            // test runners do not care about the elements:
            // because the error variants do not care about the elements!
            (TokenKind::List(m, _), TokenKind::List(n, _)) => m == n,

            // test runners cannot generate the same string: they cannot access the ParseSession
            (TokenKind::String(_), TokenKind::String(_)) => true,

            // test runners cannot generate the same identifier: they cannot access the ParseSession
            (TokenKind::Identifier(_), TokenKind::Identifier(_)) => true,

            (TokenKind::Bytes(m), TokenKind::Bytes(n)) => m == n,
            (TokenKind::FormattedString(_), TokenKind::FormattedString(_)) => true,

            _ => false,
        }
    }
}
