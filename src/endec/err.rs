pub enum EndecError {
    UnexpectedEof,
    UnexpectedByte(u8),
    Utf8Error,
}