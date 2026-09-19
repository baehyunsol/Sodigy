use crate::{Bytecode, GlobalLabel, Memory, Session, lower_expr};
use sodigy_mir as mir;
use sodigy_span::Span;
use sodigy_string::InternedString;

/// It's for top-level let statements. It's like a function with no parameters.
/// It evaluates the value, stores it to `Memory::Global(def_span)` and return the value.
///
/// It doesn't check whether it's already initialized or not. That's caller's responsibility.
#[derive(Clone, Debug)]
pub struct Let {
    pub name: InternedString,
    pub name_span: Span,
    pub bytecodes: Vec<Bytecode>,
}

impl Let {
    pub fn from_mir(mir_let: &mir::Let, session: &mut Session) -> Let {
        session.label_counter = 0;
        session.ssa_counter = 0;
        let mut bytecodes = vec![
            Bytecode::Label(session.get_local_label()),
        ];
        let return_ssa = session.get_ssa();

        lower_expr(
            &mir_let.value,
            session,
            &mut bytecodes,
            Memory::SSA(return_ssa),
            /* is_tail_call: */ false,
        );
        bytecodes.push(Bytecode::StoreGlobal {
            src: return_ssa,
            dst: GlobalLabel::new(mir_let.name_span.hash()),
        });
        bytecodes.push(Bytecode::Return(return_ssa));

        Let {
            name: mir_let.name,
            name_span: mir_let.name_span.clone(),
            bytecodes,
        }
    }
}
