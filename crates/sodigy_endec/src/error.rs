#[derive(Debug)]
pub enum EndecError {
    Eof,
    Overflow,
    FromUtf8Error,
    InvalidEnumVariant { variant_index: u8 },
    InvalidInternedString,
    InvalidInternedNumeric,
}

impl From<std::string::FromUtf8Error> for EndecError {
    fn from(_: std::string::FromUtf8Error) -> Self {
        EndecError::FromUtf8Error
    }
}
