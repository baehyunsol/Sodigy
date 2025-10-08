use crate::Tokens;
use sodigy_error::Error;
use sodigy_number::InternedNumber;

#[derive(Clone, Debug)]
pub enum Pattern {
    Number(InternedNumber),
}

impl<'t> Tokens<'t> {
    pub fn parse_pattern(&mut self) -> Result<Pattern, Vec<Error>> {
        panic!("TODO: {:?}", self.tokens)
    }
}
