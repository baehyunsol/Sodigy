use crate::{Bytecode, Session};
use sodigy_mir as mir;
use sodigy_span::Span;
use sodigy_string::InternedString;

pub struct Let {
    pub name: InternedString,
    pub name_span: Span,
    pub bytecodes: Vec<Bytecode>,
}

impl Let {
    pub fn from_mir(mir_let: &mir::Let, session: &mut Session) -> Let {
        todo!()
    }
}
