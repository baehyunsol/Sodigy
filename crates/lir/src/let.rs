use crate::{Bytecode, Register, Session, lower_mir_expr};
use sodigy_mir as mir;
use sodigy_span::Span;
use sodigy_string::InternedString;

// It's only for top-level `let` statements.
#[derive(Clone, Debug)]
pub struct Let {
    pub name: InternedString,
    pub name_span: Span,

    // When you eval this, it'll push the value to `Register::Const` and return to caller.
    // Please make sure to push the return address to the call stack before evaluating this!
    pub bytecode: Vec<Bytecode>,

    // After calling `session.make_labels_static`, every object will be mapped to a `Label::Static(id)`.
    // This is the id of the label.
    pub label_id: Option<u32>,
}

impl Let {
    // It's only for top-level `let` statements.
    pub fn from_mir(mir_let: &mir::Let, session: &mut Session) -> Let {
        session.label_counter = 0;
        let mut bytecode = vec![];
        lower_mir_expr(&mir_let.value, session, &mut bytecode, false /* is_tail_call */);
        bytecode.push(Bytecode::Push {
            src: Register::Return,
            dst: Register::Const(mir_let.name_span),
        });
        bytecode.push(Bytecode::Return);

        Let {
            name: mir_let.name,
            name_span: mir_let.name_span,
            bytecode,
            label_id: None,
        }
    }
}
