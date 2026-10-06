use crate::ObjectFile;

mod error;
mod lex;
mod token;

#[cfg(test)]
mod tests;

pub use error::ParseError;
use token::{Keyword, Token};

pub fn parse(b: &[u8]) -> Result<ObjectFile, ParseError> {
    let [data, code, entries] = lex::lex(b)?;
    todo!()
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Section {
    Data,
    Code,
    Label,
}

