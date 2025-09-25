use crate::{Decorator, DocComment, Tokens};
use sodigy_error::Error;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Enum {
    pub name: InternedString,
    pub name_span: Span,
    pub doc_comment: Option<DocComment>,
    pub decorators: Vec<Decorator>,
}

impl<'t> Tokens<'t> {
    pub fn parse_enum(&mut self) -> Result<Enum, Vec<Error>> {
        todo!();
    }
}
