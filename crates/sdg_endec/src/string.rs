use super::{Endec, EndecError};

impl Endec for String {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.as_bytes().to_vec().encode(buffer);
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        Ok(String::from_utf8(Vec::<u8>::decode(buffer, index)?)?)
    }
}

use std::string::FromUtf8Error;

impl From<FromUtf8Error> for EndecError {
    fn from(_: FromUtf8Error) -> Self {
        EndecError::Utf8Error
    }
}