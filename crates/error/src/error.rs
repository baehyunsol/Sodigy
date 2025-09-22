use sodigy_span::Span;
use sodigy_token::ErrorToken;

#[derive(Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,
}

#[derive(Clone)]
pub enum ErrorKind {
    InvalidNumberLiteral,
    UnterminatedBlockComment,
    UnexpectedToken {
        expected: ErrorToken,
        got: ErrorToken,
    },
    UnexpectedEof {
        expected: ErrorToken,
    },
    UnclosedDelimiter(u8),
}
