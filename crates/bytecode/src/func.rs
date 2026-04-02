use crate::{Bytecode, Memory, Session, lower_expr};
use sodigy_mir as mir;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Func {
    pub name: InternedString,
    pub name_span: Span,
    pub params: usize,
    pub bytecodes: Vec<Bytecode>,
}

impl Func {
    pub fn from_mir(mir_func: &mir::Func, session: &mut Session) -> Func {
        session.label_counter = 0;
        session.ssa_map = HashMap::new();
        let mut bytecodes = vec![];

        for (i, param) in mir_func.params.iter().enumerate() {
            session.ssa_map.insert(
                param.name_span.clone(),
                i as u32,
            );
        }

        session.ssa_counter = mir_func.params.len() as u32;
        lower_expr(
            &mir_func.value,
            session,
            &mut bytecodes,
            Memory::Return,
            /* is_tail_call: */ true,
        );

        Func {
            name: mir_func.name,
            name_span: mir_func.name_span.clone(),
            params: mir_func.params.len(),
            bytecodes,
        }
    }
}
