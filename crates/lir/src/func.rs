use crate::{Memory, Session, lower_expr};
use sodigy_mir as mir;
use std::collections::HashMap;

pub struct Func {}

impl Func {
    pub fn from_mir(mir_func: &mir::Func, session: &mut Session) -> Func {
        session.func_param_count = mir_func.params.len();
        session.label_counter = 0;
        session.local_values = HashMap::new();
        session.drop_types = HashMap::new();

        for param in mir_func.params.iter() {
            session.register_local_name(param.name_span);
        }

        let mut bytecodes = vec![];

        lower_expr(
            &mir_func.value,
            session,
            &mut bytecodes,
            Memory::Return,
            /* is_tail_call: */ true,
        );

        Func {}
    }
}
