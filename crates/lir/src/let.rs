use crate::{Bytecode, Memory, Session, lower_expr};
use sodigy_mir as mir;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

/// It's for top-level let statements.
/// When you evaluate its bytecodes, it'll evaluate itself and
/// push the result to `Memory::Global(self.def_span)`, and return.
/// 1. It doesn't check whether it's already initialized or not.
///    That's caller's responsibility.
/// 2. It returns after the evaluation. So the caller must push something
///    to the call stack.
#[derive(Clone, Debug)]
pub struct Let {
    pub name: InternedString,
    pub name_span: Span,
    pub bytecodes: Vec<Bytecode>,
}

impl Let {
    pub fn from_mir(mir_let: &mir::Let, session: &mut Session) -> Let {
        session.label_counter = 0;
        session.local_values = HashMap::new();
        let mut bytecodes = vec![];

        session.collect_local_names(&mir_let.value, 0);
        session.stack_offset = session.local_values.values().map(
            |local_value| local_value.stack_offset + 1
        ).max().unwrap_or(0);

        lower_expr(
            &mir_let.value,
            session,
            &mut bytecodes,
            Memory::Global(mir_let.name_span),
            /* is_tail_call: */ false,
        );
        bytecodes.push(Bytecode::Return);

        Let {
            name: mir_let.name,
            name_span: mir_let.name_span,
            bytecodes,
        }
    }
}
