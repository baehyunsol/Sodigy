use crate::{
    Bytecode,
    Const,
    Register,
    Session,
    lower_mir_expr,
};
use sodigy_mir::{self as mir, Intrinsic};
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Assert {
    pub name: Option<InternedString>,
    pub keyword_span: Span,

    // When you evaluate this it might 1) eprint error and panic (if the assertion is False) or 2) do nothing.
    pub bytecodes: Vec<Bytecode>,

    // After calling `session.make_labels_static`, every object will be mapped to a `Label::Static(id)`.
    // This is the id of the label.
    pub label_id: Option<u32>,
}

impl Assert {
    pub fn from_mir(mir_assert: &mir::Assert, session: &mut Session, is_top_level: bool) -> Assert {
        session.label_counter = 0;
        let mut bytecodes = vec![];
        lower_mir_expr(&mir_assert.value, session, &mut bytecodes, false /* tail_call */);

        let no_panic = session.get_tmp_label();
        bytecodes.push(Bytecode::JumpIf {
            value: Register::Return,
            label: no_panic,
        });
        bytecodes.push(Bytecode::PushConst {
            value: Const::String {
                s: mir_assert.error_message,
                binary: false,
            },
            dst: Register::Call(0),
        });
        bytecodes.push(Bytecode::Intrinsic(Intrinsic::EPrint));
        bytecodes.push(Bytecode::Pop(Register::Call(0)));
        bytecodes.push(Bytecode::Intrinsic(Intrinsic::Panic));
        bytecodes.push(Bytecode::Label(no_panic));

        if is_top_level {
            bytecodes.push(Bytecode::Intrinsic(Intrinsic::Exit));
        }

        Assert {
            name: mir_assert.name,
            keyword_span: mir_assert.keyword_span,
            bytecodes,
            label_id: None,
        }
    }
}
