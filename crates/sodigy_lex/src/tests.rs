use super::*;

impl Token {
    pub fn assert_valid_span(&self) {
        let original_code = self.span.to_utf8();

        match &self.kind {
            TokenKind::Identifier(id) => debug_assert_eq!(
                original_code,
                sodigy_intern::unintern_string(*id),
            ),
            TokenKind::Whitespace => {},
            TokenKind::Punct(p)
            | TokenKind::Grouper(p) => debug_assert_eq!(
                original_code,
                vec![*p],
            ),
            TokenKind::Comment { kind, .. } => match kind {
                CommentKind::Single => debug_assert!(original_code.starts_with(b"#")),
                CommentKind::Multi => debug_assert!(original_code.starts_with(b"#!")),
                CommentKind::Doc => debug_assert!(original_code.starts_with(b"#>")),
            },
            TokenKind::String { kind, .. } => {
                debug_assert_eq!(*original_code.first().unwrap(), *kind as u8);
                debug_assert_eq!(*original_code.last().unwrap(), *kind as u8);
            },
            TokenKind::Number(_) => debug_assert!(
                original_code[0].is_ascii_digit()
            ),
        }
    }
}
