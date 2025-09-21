use sodigy_span::Span;

pub struct Error {
    kind: ErrorKind,
    span: Span,
}

pub enum ErrorKind {
    InvalidNumberLiteral,
    UnterminatedBlockComment,
}
