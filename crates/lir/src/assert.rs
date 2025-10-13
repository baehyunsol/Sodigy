use crate::{Bytecode, Session, lower_mir_expr};
use sodigy_mir as mir;
use sodigy_span::Span;

#[derive(Clone, Debug)]
pub struct Assert {
    pub keyword_span: Span,

    // When you evaluate this, it'll push a boolean (the assertion) to
    // `Register::Return` and return. Make sure to push a return address
    // to the call stack before evaluating this!
    pub bytecode: Vec<Bytecode>,

    // After calling `session.make_labels_static`, every object will be mapped to a `Label::Static(id)`.
    // This is the id of the label.
    pub label_id: Option<u32>,
}

impl Assert {
    pub fn from_mir(mir_assert: &mir::Assert, session: &mut Session) -> Assert {
        session.label_counter = 0;
        let mut bytecode = vec![];
        lower_mir_expr(&mir_assert.value, session, &mut bytecode, false /* tail_call */);
        bytecode.push(Bytecode::Return);

        Assert {
            keyword_span: mir_assert.keyword_span,
            bytecode,
            label_id: None,
        }
    }
}
