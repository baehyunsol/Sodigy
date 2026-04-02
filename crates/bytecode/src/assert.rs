use crate::{
    Bytecode,
    DebugInfoKind,
    Memory,
    Session,
    Value,
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

        let mut bytecodes = vec![];
        let mut debug_info_count = 0;

        let span_ssa = session.get_ssa();
        bytecodes.push(Bytecode::Const {
            value: Value::Span(mir_assert.keyword_span.clone()),
            dst: Memory::SSA(span_ssa),
        });
        bytecodes.push(Bytecode::PushDebugInfo {
            kind: DebugInfoKind::AssertionKeywordSpan,
            src: Memory::SSA(span_ssa),
        });
        debug_info_count += 1;

        let name = match &mir_assert.name {
            Some(name) => *name,
            None => intern_string(b"unnamed-assertion", &session.intermediate_dir).unwrap(),
        };
        let name_ssa = session.get_ssa();
        bytecodes.push(Bytecode::Const {
            value: session.string_to_value(name, /* binary: */ false),
            dst: Memory::SSA(name_ssa),
        });
        bytecodes.push(Bytecode::PushDebugInfo {
            kind: DebugInfoKind::AssertionName,
            src: Memory::SSA(name_ssa),
        });
        debug_info_count += 1;

        let value_ssa = session.get_ssa();
        lower_expr(
            &mir_assert.value,
            session,
            &mut bytecodes,
            Memory::SSA(value_ssa),
            /* is_tail_call: */ false,
        );

        let no_panic = session.get_local_label();
        bytecodes.push(Bytecode::JumpIf {
            value: Memory::SSA(value_ssa),
            label: no_panic.clone(),
        });

        // We don't pop_debug_info for error notes because notes are evaluated only if the assertion has failed.
        if let (Some(note), Some(note_decorator_span)) = (&mir_assert.note, &mir_assert.note_decorator_span) {
            // If it panics while evaluating `note`, the runtime will see the
            // `NoteDecoratorSpan` and throw an according error message.
            let span_ssa = session.get_ssa();
            bytecodes.push(Bytecode::Const {
                value: Value::Span(note_decorator_span.clone()),
                dst: Memory::SSA(span_ssa),
            });
            bytecodes.push(Bytecode::PushDebugInfo {
                kind: DebugInfoKind::AssertionNoteDecoratorSpan,
                src: Memory::SSA(span_ssa),
            });

            let note_ssa = session.get_ssa();
            lower_expr(
                note,
                session,
                &mut bytecodes,
                Memory::SSA(note_ssa),
                /* is_tail_call: */ false,
            );
            bytecodes.push(Bytecode::PushDebugInfo {
                kind: DebugInfoKind::AssertionNote,
                src: Memory::SSA(note_ssa),
            });
        }

        // When it panics, the runtime will see the values in the AssertionMetadata stack
        // and throw an error message.
        bytecodes.push(Bytecode::Intrinsic {
            intrinsic: Intrinsic::Panic,
            args: vec![],
            dst: Memory::Return,  // don't care
        });
        bytecodes.push(Bytecode::Label(no_panic.clone()));

        for _ in 0..debug_info_count {
            bytecodes.push(Bytecode::PopDebugInfo);
        }

        if is_top_level {
            bytecodes.push(Bytecode::Intrinsic {
                intrinsic: Intrinsic::Exit,
                args: vec![],
                dst: Memory::Return,  // don't care
            });
        }

        Assert {
            name,
            keyword_span: mir_assert.keyword_span.clone(),
            bytecodes,
        }
    }
}
