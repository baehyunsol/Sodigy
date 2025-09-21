use sodigy_keyword::Keyword;
use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;

pub struct Token {
    kind: TokenKind,
    span: Span,
}

pub enum TokenKind {
    Keyword(Keyword),
    Identifier(InternedString),
    Number(InternedNumber),
}
