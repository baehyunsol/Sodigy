use crate::Type;
use sodigy_error::Error;
use sodigy_hir::{self as hir, FuncArgDef};
use sodigy_span::Span;
use std::collections::HashMap;

pub struct Session {
    pub func_args: HashMap<Span, Vec<FuncArgDef<Type>>>,
    pub errors: Vec<Error>,
}

impl Session {
    pub fn from_hir_session(hir_session: &hir::Session) -> Session {
        Session {
            func_args: todo!(),
            errors: vec![],
        }
    }
}
