use crate::{Bytecode, Memory, Session, lower_expr};
use sodigy_mir as mir;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Let {
    pub name: InternedString,
    pub name_span: Span,
    pub bytecodes: Vec<Bytecode>,
}

impl Let {
    pub fn from_mir(mir_let: &mir::Let, session: &mut Session) -> Let {
        let mut bytecodes = vec![];

        lower_expr(
            &mir_let.value,
            session,
            &mut bytecodes,
            Memory::Global(mir_let.name_span),
            /* is_tail_call: */ false,
        );

        Let {
            name: mir_let.name,
            name_span: mir_let.name_span,
            bytecodes,
        }
    }
}
