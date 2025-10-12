use crate::{Assert, Enum, Func, Let, PRELUDES, Struct};
use sodigy_error::{Error, Warning};
use sodigy_fs_api::join;
use sodigy_name_analysis::{Namespace, UseCount};
use sodigy_span::Span;
use sodigy_string::intern_string;

pub struct Session {
    pub intern_str_map_dir: String,
    pub intern_num_map_dir: String,
    pub name_stack: Vec<Namespace>,
    pub is_evaluating_assertion: bool,

    // Top-level declarations are stored here.
    // Also, many inline declarations are stored here (so that inline blocks get simpler).
    pub lets: Vec<Let>,
    pub funcs: Vec<Func>,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,
    pub asserts: Vec<Assert>,

    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn new(intern_map_dir: &str) -> Self {
        let intern_str_map_dir = join(intern_map_dir, "str").unwrap();
        let prelude_namespace = Namespace::Block {
            names: PRELUDES.iter().map(
                |(name, kind)| (
                    intern_string(name, &intern_str_map_dir).unwrap(),
                    (
                        Span::Prelude(intern_string(name, &intern_str_map_dir).unwrap()),
                        *kind,
                        UseCount::new(),
                    ),
                )
            ).collect(),
        };

        Session {
            intern_str_map_dir,
            intern_num_map_dir: join(intern_map_dir, "num").unwrap(),
            name_stack: vec![prelude_namespace],
            is_evaluating_assertion: false,
            lets: vec![],
            funcs: vec![],
            structs: vec![],
            enums: vec![],
            asserts: vec![],
            errors: vec![],
            warnings: vec![],
        }
    }
}
