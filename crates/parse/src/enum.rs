use crate::{Attribute, Tokens};
use sodigy_error::Error;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Enum {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub attribute: Attribute,
}

impl<'t> Tokens<'t> {
    pub fn parse_enum(&mut self) -> Result<Enum, Vec<Error>> {
        todo!();
    }
}
