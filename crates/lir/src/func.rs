use crate::{Bytecode, DropType, LocalValue, Memory, Session, lower_expr};
use sodigy_mir as mir;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Func {
    pub name: InternedString,
    pub name_span: Span,
    pub bytecodes: Vec<Bytecode>,
}

impl Func {
    pub fn from_mir(mir_func: &mir::Func, session: &mut Session) -> Func {
        session.label_counter = 0;
        session.local_values = HashMap::new();
        let mut bytecodes = vec![];

        for (i, param) in mir_func.params.iter().enumerate() {
            session.local_values.insert(
                param.name_span,
                LocalValue {
                    stack_offset: i,
                    dropped: false,

                    // TODO: drop value!!!
                    drop_type: DropType::Scalar,
                },
            );
        }

        session.collect_local_names(&mir_func.value, mir_func.params.len());
        session.stack_offset = session.local_values.values().map(
            |local_value| local_value.stack_offset + 1
        ).max().unwrap_or(0);

        lower_expr(
            &mir_func.value,
            session,
            &mut bytecodes,
            Memory::Return,
            /* is_tail_call: */ true,
        );

        Func {
            name: mir_func.name,
            name_span: mir_func.name_span,
            bytecodes,
        }
    }
}
