use crate::ObjectFile;

mod error;
mod lex;

pub use error::Error;
use lex::lex;

pub fn parse(bytes: &[u8]) -> Result<ObjectFile, Error> {
    let tokens = lex(bytes)?;
    todo!()
}

