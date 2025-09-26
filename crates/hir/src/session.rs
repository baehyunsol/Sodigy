use crate::Namespace;
use sodigy_error::Error;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::{HashMap, HashSet};

pub struct Session {
    pub curr_func_args: HashMap<InternedString, (usize, Span)>,
    pub name_stack: Vec<Namespace>,
    pub foreign_names: HashSet<(InternedString, Span)>,
    pub errors: Vec<Error>,
}

impl Session {
    pub fn new() -> Self {
        Session {
            curr_func_args: HashMap::new(),
            name_stack: vec![],
            foreign_names: HashSet::new(),
            errors: vec![],
        }
    }
}
