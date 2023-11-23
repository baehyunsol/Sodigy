#[derive(Debug)]
pub enum EndecErr {
    Eof,
    Overflow,
    FromUtf8Error,
    InvalidEnumVariant { variant_index: u8 },
    InvalidInternedString,
    InvalidInternedNumeric,
}

impl From<std::string::FromUtf8Error> for EndecErr {
    fn from(_: std::string::FromUtf8Error) -> Self {
        EndecErr::FromUtf8Error
    }
}
