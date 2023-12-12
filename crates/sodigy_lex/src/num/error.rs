use super::super::error::ExpectedChars;

#[derive(Debug, PartialEq)]
pub enum ParseNumberError {
    UnfinishedNumLiteral(ExpectedChars),
}
