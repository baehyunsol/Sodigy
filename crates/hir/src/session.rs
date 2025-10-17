use crate::{Assert, Enum, Func, Let, PRELUDES, Struct};
use sodigy_error::{Error, Warning};
use sodigy_fs_api::join;
use sodigy_name_analysis::{Namespace, UseCount};
use sodigy_parse::Session as ParseSession;
use sodigy_span::Span;
use sodigy_string::intern_string;

pub struct Session {
    pub intermediate_dir: String,
    pub name_stack: Vec<Namespace>,

    // `func_default_values.last()` has the default values of functions
    // in the current block.
    // If it enters a new block, it pushes `vec![]` to `func_default_values`.
    // When it leaves a block, it pops `let` statements from `func_default_values`
    // and pushes them to the current block.
    pub func_default_values: Vec<Vec<Let>>,

    // The expr/func/block it's lowering only exists in debug context.
    pub is_in_debug_context: bool,

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
    pub fn from_parse_session(parse_session: &ParseSession) -> Self {
        let prelude_namespace = Namespace::Block {
            names: PRELUDES.iter().map(
                |(name, kind)| (
                    intern_string(name, &parse_session.intermediate_dir).unwrap(),
                    (
                        Span::Prelude(intern_string(name, &parse_session.intermediate_dir).unwrap()),
                        *kind,
                        UseCount::new(),
                    ),
                )
            ).collect(),
        };

        Session {
            intermediate_dir: parse_session.intermediate_dir.to_string(),
            name_stack: vec![prelude_namespace],
            func_default_values: vec![],
            is_in_debug_context: false,
            lets: vec![],
            funcs: vec![],
            structs: vec![],
            enums: vec![],
            asserts: vec![],
            errors: parse_session.errors.clone(),
            warnings: parse_session.warnings.clone(),
        }
    }

    pub fn push_func_default_value(&mut self, default_value: Let) {
        self.func_default_values.last_mut().unwrap().push(default_value);
    }
}
