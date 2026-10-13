use sodigy_span::Span;

pub enum Keyword {
    If,
    Else,
    Jump,
    Return,
    Label,
    With,
}

pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub enum TokenKind {
    Keyword(Keyword),
}

