#[derive(Clone, Debug)]
pub enum DecodeError {
    RemainingBytes,
    UnexpectedEof,
    InvalidEnumVariant(u8),
    InvalidUtf8,
}
