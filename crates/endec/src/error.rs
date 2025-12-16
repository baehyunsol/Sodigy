#[derive(Clone, Debug)]
pub enum DecodeError {
    RemainingBytes,
    UnexpectedEof,
    InvalidEnumVariant(u8),
    InvalidLargeEnumVariant(u32),
    InvalidUtf8,
    InvalidUnicodePoint(u32),
}
