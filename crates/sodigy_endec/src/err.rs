#[derive(Debug)]
pub enum EndecErr {
    Eof,
    Overflow,
    FromUtf8Error,
    InvalidEnumVariant { variant_index: u8 },
}

impl From<std::string::FromUtf8Error> for EndecErr {
    fn from(e: std::string::FromUtf8Error) -> Self {
        EndecErr::FromUtf8Error
    }
}
