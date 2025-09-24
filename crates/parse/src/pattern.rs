use crate::Tokens;
use sodigy_error::Error;

#[derive(Clone, Debug)]
pub struct Pattern;

impl<'t> Tokens<'t> {
    pub fn parse_pattern(&mut self) -> Result<Pattern, Vec<Error>> {
        todo!()
    }
}
