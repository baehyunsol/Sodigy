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
use std::collections::HashMap;

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
            session.local_values = HashMap::new();

            session.collect_local_names(&mir_assert.value, 0);
            session.stack_offset = session.local_values.values().map(
                |local_value| local_value.stack_offset + 1
            ).max().unwrap_or(0);
        }

        let mut bytecodes = vec![];
        let mut debug_info_count = 0;

        let name = match &mir_assert.name {
            Some(name) => *name,
            None => intern_string(b"unnamed-assertion", &session.intermediate_dir).unwrap(),
        };
        bytecodes.push(Bytecode::Const {
            value: Value::Span(mir_assert.keyword_span),
            dst: Memory::Return,
        });
        bytecodes.push(Bytecode::PushDebugInfo {
            kind: DebugInfoKind::AssertionKeywordSpan,
            src: Memory::Return,
        });
        debug_info_count += 1;

        bytecodes.push(Bytecode::Const {
            value: session.string_to_value(name, /* binary: */ false),
            dst: Memory::Return,
        });
        bytecodes.push(Bytecode::PushDebugInfo {
            kind: DebugInfoKind::AssertionName,
            src: Memory::Return,
        });
        debug_info_count += 1;

        if let (Some(note), Some(note_decorator_span)) = (&mir_assert.note, &mir_assert.note_decorator_span) {
            // If it panics while evaluating `note`, the runtime will see the
            // `NoteDecoratorSpan` and throw an according error message.
            bytecodes.push(Bytecode::Const {
                value: Value::Span(*note_decorator_span),
                dst: Memory::Return,
            });
            bytecodes.push(Bytecode::PushDebugInfo {
                kind: DebugInfoKind::AssertionNoteDecoratorSpan,
                src: Memory::Return,
            });
            debug_info_count += 1;

            lower_expr(
                note,
                session,
                &mut bytecodes,
                Memory::Return,
                /* is_tail_call: */ false,
            );
            bytecodes.push(Bytecode::PushDebugInfo {
                kind: DebugInfoKind::AssertionNote,
                src: Memory::Return,
            });
            debug_info_count += 1;
        }

        lower_expr(
            &mir_assert.value,
            session,
            &mut bytecodes,
            Memory::Return,
            /* is_tail_call: */ false,
        );

        let no_panic = session.get_local_label();
        bytecodes.push(Bytecode::JumpIf {
            value: Memory::Return,
            label: no_panic,
        });

        // When it panics, the runtime will see the values in the AssertionMetadata stack
        // and throw an error message.
        bytecodes.push(Bytecode::Intrinsic {
            intrinsic: Intrinsic::Panic,
            stack_offset: 0,  // don't care
            dst: Memory::Return,  // don't care
        });
        bytecodes.push(Bytecode::Label(no_panic));

        for _ in 0..debug_info_count {
            bytecodes.push(Bytecode::PopDebugInfo);
        }

        if is_top_level {
            bytecodes.push(Bytecode::Intrinsic {
                intrinsic: Intrinsic::Exit,
                stack_offset: 0,  // don't care
                dst: Memory::Return,  // don't care
            });
        }

        Assert {
            name,
            keyword_span: mir_assert.keyword_span,
            bytecodes,
        }
    }
}
