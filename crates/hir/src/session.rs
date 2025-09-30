use crate::{Enum, Func, Let, PRELUDES, Struct};
use sodigy_error::{Error, Warning};
use sodigy_name_analysis::{Namespace, NamespaceKind};
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};
use std::collections::{HashMap, HashSet};

pub struct Session {
    pub name_stack: Vec<Namespace>,

    // Top-level declarations are stored here.
    // Also, many inline declarations are stored here (so that inline blocks get simpler).
    pub lets: Vec<Let>,
    pub funcs: Vec<Func>,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,

    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn new() -> Self {
        let prelude_namespace = Namespace::Block {
            names: PRELUDES.iter().map(
                |(name, kind)| (
                    intern_string(name),
                    (
                        Span::Prelude(intern_string(name)),
                        *kind,
                        0,
                    ),
                )
            ).collect(),
        };

        Session {
            name_stack: vec![prelude_namespace],
            lets: vec![],
            funcs: vec![],
            structs: vec![],
            enums: vec![],
            errors: vec![],
            warnings: vec![],
        }
    }
}
