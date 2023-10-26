use super::super::err::ExpectedChars;

#[derive(Debug, PartialEq)]
pub enum ParseNumberError {
    UnfinishedNumLiteral(ExpectedChars),
}
