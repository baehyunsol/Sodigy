use crate::{
    Alias,
    Assert,
    Enum,
    Func,
    Let,
    Module,
    Struct,
    Use,
    prelude::prelude_namespace,
};
use sodigy_error::{Error, Warning};
use sodigy_name_analysis::Namespace;
use sodigy_parse::Session as ParseSession;
use sodigy_session::Session as SodigySession;
use sodigy_span::Span;

pub struct Session {
    pub intermediate_dir: String,
    pub main_func: Option<Span>,
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
    pub aliases: Vec<Alias>,

    // it includes top-level and inline `use` statements,
    // so that it knows which files to look for.
    pub uses: Vec<Use>,

    // modules are always top-level
    pub modules: Vec<Module>,

    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn from_parse_session(parse_session: &ParseSession) -> Self {
        Session {
            intermediate_dir: parse_session.intermediate_dir.to_string(),
            main_func: parse_session.main_func,
            name_stack: vec![prelude_namespace(&parse_session.intermediate_dir)],
            func_default_values: vec![],
            is_in_debug_context: false,
            lets: vec![],
            funcs: vec![],
            structs: vec![],
            enums: vec![],
            aliases: vec![],
            asserts: vec![],
            uses: vec![],
            modules: vec![],
            errors: parse_session.errors.clone(),
            warnings: parse_session.warnings.clone(),
        }
    }

    pub fn push_func_default_value(&mut self, default_value: Let) {
        self.func_default_values.last_mut().unwrap().push(default_value);
    }
}

impl SodigySession for Session {
    fn get_errors(&self) -> &[Error] {
        &self.errors
    }

    fn get_warnings(&self) -> &[Warning] {
        &self.warnings
    }

    fn get_intermediate_dir(&self) -> &str {
        &self.intermediate_dir
    }
}
