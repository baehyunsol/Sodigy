use crate::ObjectFile;
use sodigy_error::Error;
use sodigy_file::File;

mod lex;
mod token;

use lex::lex;
use token::{
    Command,
    Keyword,
    SectionName,
    Token,
    TokenKind,
};

pub fn parse(bytes: &[u8], file: File, intermediate_dir: &str) -> Result<ObjectFile, Error> {
    let tokens = lex(bytes, file, intermediate_dir)?;
    todo!()
}

