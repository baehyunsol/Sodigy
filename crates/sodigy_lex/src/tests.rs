use super::*;
use sodigy_test::{sodigy_assert, sodigy_assert_eq};

impl Token {
    pub fn assert_valid_span(&self) {
        let original_code = self.span.to_utf8();

        match &self.kind {
            TokenKind::Identifier(id) => sodigy_assert_eq!(
                original_code,
                sodigy_intern::unintern_string(*id),
            ),
            TokenKind::Whitespace => {},
            TokenKind::Punct(p)
            | TokenKind::Grouper(p) => sodigy_assert_eq!(
                original_code,
                vec![*p],
            ),
            TokenKind::Comment { kind, .. } => match kind {
                CommentKind::Single => sodigy_assert!(original_code.starts_with(b"#")),
                CommentKind::Multi => sodigy_assert!(original_code.starts_with(b"##!")),
                CommentKind::Doc => sodigy_assert!(original_code.starts_with(b"##>")),
            },
            TokenKind::String { kind, .. } => {
                sodigy_assert_eq!(*original_code.first().unwrap(), *kind as u8);
                sodigy_assert_eq!(*original_code.last().unwrap(), *kind as u8);
            },
            TokenKind::Number(_) => sodigy_assert!(
                original_code[0].is_ascii_digit()
            ),
        }
    }
}
