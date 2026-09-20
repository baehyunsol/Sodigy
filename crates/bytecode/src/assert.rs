use crate::{
    Bytecode,
    InternedValue,
    Memory,
    Session,
    lower_expr,
};
use sodigy_mir::{self as mir, Intrinsic};
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};

#[derive(Clone, Debug)]
pub struct Assert {
    pub keyword_span: Span,

    // If the user didn't specify, it's "unnamed-assertion".
    pub name: InternedString,

    pub bytecodes: Vec<Bytecode>,
}

impl Assert {
    pub fn from_mir(mir_assert: &mir::Assert, session: &mut Session, is_top_level: bool) -> Assert {
        if is_top_level {
            session.label_counter = 0;
            session.ssa_counter = 0;
        }

        let mut bytecodes = vec![
            Bytecode::Label(session.get_local_label()),
        ];

        let name = match &mir_assert.name {
            Some(name) => *name,
            None => intern_string(b"unnamed-assertion", &session.intermediate_dir).unwrap(),
        };

        let value_ssa = session.get_ssa();
        lower_expr(
            &mir_assert.value,
            session,
            &mut bytecodes,
            Memory::SSA(value_ssa),
            /* is_tail_call: */ false,
        );

        let no_panic = session.get_local_label();
        let do_panic = session.get_local_label();
        bytecodes.push(Bytecode::JumpIf {
            value: value_ssa,
            t: no_panic,
            f: do_panic,

            // I don't think we need debug_info for this because we already have the span of the `assert` keyword.
            debug_info: None,
        });
        bytecodes.push(Bytecode::Label(do_panic));

        // We don't pop_debug_info for error notes because notes are evaluated only if the assertion has failed.
        if let (Some(note), Some(note_decorator_span)) = (&mir_assert.note, &mir_assert.note_decorator_span) {
            let note_ssa = session.get_ssa();
            lower_expr(
                note,
                session,
                &mut bytecodes,
                Memory::SSA(note_ssa),
                /* is_tail_call: */ false,
            );
            // TODO: dump note to stderr
        }

        let status_code = session.get_ssa();
        bytecodes.push(Bytecode::Const {
            value: InternedValue::Scalar(22),
            dst: Memory::SSA(status_code),
            debug_info: None,
        });
        let null_ssa = session.get_ssa();
        bytecodes.push(Bytecode::Intrinsic {
            intrinsic: Intrinsic::Exit,
            args: vec![status_code],
            dst: Memory::SSA(null_ssa),
            debug_info: None,
        });
        bytecodes.push(Bytecode::Label(no_panic.clone()));

        if is_top_level {
            let status_code = session.get_ssa();
            bytecodes.push(Bytecode::Const {
                value: InternedValue::Scalar(0),
                dst: Memory::SSA(status_code),
                debug_info: None,
            });
            let null_ssa = session.get_ssa();
            bytecodes.push(Bytecode::Intrinsic {
                intrinsic: Intrinsic::Exit,
                args: vec![status_code],
                dst: Memory::SSA(null_ssa),
                debug_info: None,
            });

            // This bytecode is unreachable.
            // But without this, `to_basic_blocks` won't work.
            bytecodes.push(Bytecode::Return(crate::SSA(0)));
        }

        Assert {
            name,
            keyword_span: mir_assert.keyword_span.clone(),
            bytecodes,
        }
    }
}
