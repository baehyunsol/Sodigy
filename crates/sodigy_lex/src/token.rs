pub struct Token {
    kind: TokenKind,
    span: Span,
}

pub enum TokenKind {
    Integer(InternedNumber),
}
