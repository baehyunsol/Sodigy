use crate::{Enum, Func, Let, PRELUDES, Struct};
use sodigy_error::Error;
use sodigy_name_analysis::{Namespace, NamespaceKind};
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};
use std::collections::{HashMap, HashSet};

pub struct Session {
    pub curr_func_args: HashMap<InternedString, (usize, Span)>,
    pub name_stack: Vec<Namespace>,
    pub foreign_names: HashSet<(InternedString, Span)>,

    // Top-level declarations are stored here.
    // Also, many inline declarations are stored here (so that inline blocks get simpler).
    pub lets: Vec<Let>,
    pub funcs: Vec<Func>,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,

    pub errors: Vec<Error>,
}

impl Session {
    pub fn new() -> Self {
        let prelude_namespace = Namespace {
            kind: NamespaceKind::Prelude,
            names: PRELUDES.iter().map(
                |(name, kind)| (
                    intern_string(name),
                    (
                        Span::Prelude(intern_string(name)),
                        *kind,
                    ),
                )
            ).collect(),
        };

        Session {
            curr_func_args: HashMap::new(),
            name_stack: vec![prelude_namespace],
            foreign_names: HashSet::new(),
            lets: vec![],
            funcs: vec![],
            structs: vec![],
            enums: vec![],
            errors: vec![],
        }
    }
}
