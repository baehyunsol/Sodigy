use crate::{Decorator, DocComment, Expr, Tokens};
use sodigy_error::Error;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Debug)]
pub struct Let {
    name: InternedString,
    name_span: Span,
    r#type: Option<Expr>,
    value: Expr,
    pub doc_comment: Option<DocComment>,
    pub decorators: Vec<Decorator>,
}

impl<'t> Tokens<'t> {
    pub fn parse_let(&mut self) -> Result<Let, Vec<Error>> {
        todo!()
    }
}
