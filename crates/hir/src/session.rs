use crate::{
    Alias,
    Assert,
    Enum,
    Func,
    Let,
    LetOrigin,
    Module,
    Struct,
    Use,
    prelude::prelude_namespace,
};
use sodigy_error::{Error, Warning};
use sodigy_name_analysis::{NameKind, Namespace};
use sodigy_parse::Session as ParseSession;
use sodigy_session::Session as SodigySession;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

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
    pub aliases: Vec<Alias>,

    // it includes top-level and inline `use` statements,
    // so that it knows which files to look for.
    pub uses: Vec<Use>,

    // modules are always top-level
    pub modules: Vec<Module>,

    // inter-hir will collect this
    pub lang_items: HashMap<String, Span>,

    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn from_parse_session(parse_session: &ParseSession) -> Self {
        Session {
            intermediate_dir: parse_session.intermediate_dir.to_string(),
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
            lang_items: HashMap::new(),
            errors: parse_session.errors.clone(),
            warnings: parse_session.warnings.clone(),
        }
    }

    // TODO: more fine-grained filtering for visibility
    pub fn iter_public_names(&self) -> impl Iterator<Item = (InternedString, Span, NameKind)> {
        self.lets.iter().filter(
            |r#let| r#let.visibility.is_public()
        ).map(
            |r#let| (r#let.name, r#let.name_span, NameKind::Let { is_top_level: r#let.origin == LetOrigin::TopLevel })
        ).chain(
            self.funcs.iter().filter(
                |func| func.visibility.is_public()
            ).map(
                |func| (func.name, func.name_span, NameKind::Func)
            )
        ).chain(
            self.structs.iter().filter(
                |r#struct| r#struct.visibility.is_public()
            ).map(
                |r#struct| (r#struct.name, r#struct.name_span, NameKind::Struct)
            )
        )
        // TODO: `enum` is not worked yet
        // .chain(
        //     self.enums.iter().filter(
        //         |r#enum| r#enum.visibility.is_public()
        //     ).map(
        //         |r#enum| (r#enum.name, r#enum.name_span, NameKind::Enum)
        //     )
        // )
        .chain(
            self.aliases.iter().filter(
                // TODO
                // |alias| alias.visibility.is_public()
                |_| true
            ).map(
                |alias| (alias.name, alias.name_span, NameKind::Alias)
            )
        ).chain(
            self.uses.iter().filter(
                // TODO
                // |r#use| r#use.visibility.is_public()
                |_| true
            ).map(
                |r#use| (r#use.name, r#use.name_span, NameKind::Use)
            )
        ).chain(
            self.modules.iter().filter(
                |module| module.visibility.is_public()
            ).map(
                |module| (module.name, module.name_span, NameKind::Use)
            )
        )
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
