use crate::{
    Alias,
    Assert,
    AssociatedItem,
    AttributeRule,
    AttributeRuleKey,
    BlockSession,
    CapturedNames,
    Enum,
    Expr,
    Func,
    Let,
    LetOrigin,
    Module,
    Poly,
    Struct,
    TrivialLet,
    TypeAssertion,
    Use,
    prelude::prelude_namespace,
};
use sodigy_error::{Error, Warning, WarningKind};
use sodigy_name_analysis::{Counter, NameKind, Namespace, UseCount};
use sodigy_parse::Session as ParseSession;
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::{InternedString, intern_string};
use std::collections::hash_map::{Entry, HashMap};

pub struct Session {
    pub intermediate_dir: String,
    pub name_stack: Vec<Namespace>,
    pub block_stack: Vec<BlockSession>,

    // It'd be too expensive to instantiate a rule each time...
    pub attribute_rule_cache: HashMap<AttributeRuleKey, AttributeRule>,

    // `is_in_debug_context` might change in a file, but `is_std` doesn't change inside a file.
    pub is_in_debug_context: bool,
    pub is_std: bool,

    pub nested_pipeline_depth: usize,

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

    // collected all the `#[assert_type(..)]` in this module
    pub type_assertions: Vec<TypeAssertion>,

    pub associated_items: Vec<AssociatedItem>,

    // key: name_span of `let`
    pub trivial_lets: HashMap<Span, TrivialLet>,

    // after ast is lowered to hir, the session will walk the tree and
    // replace `Expr::Ident(x)` with `Expr::Closure { fp: x, captures: captured_names.locals }`
    pub closures: HashMap<Span /* def_span of lambda */, CapturedNames>,

    // inter-hir will collect these
    pub lang_items: HashMap<String, Span>,
    pub polys: HashMap<Span, Poly>,
    pub poly_impls: Vec<(Expr /* path to the poly */, Span /* def_span of implementation */)>,

    // TODO: attribute for the current module

    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn from_parse_session(parse_session: &ParseSession) -> Self {
        let std_and_lib = Namespace::Block {
            names: vec![
                (
                    intern_string(b"std", &parse_session.intermediate_dir).unwrap(),
                    (Span::Std, NameKind::Module, UseCount::new()),
                ),
                (
                    intern_string(b"lib", &parse_session.intermediate_dir).unwrap(),
                    (Span::Lib, NameKind::Module, UseCount::new()),
                ),
            ].into_iter().collect::<HashMap<_, _>>(),
        };
        let name_stack = if parse_session.is_std {
            vec![std_and_lib]
        } else {
            vec![
                std_and_lib,
                prelude_namespace(&parse_session.intermediate_dir),
            ]
        };

        Session {
            intermediate_dir: parse_session.intermediate_dir.to_string(),
            name_stack,
            block_stack: vec![],
            attribute_rule_cache: HashMap::new(),
            is_in_debug_context: false,
            is_std: parse_session.is_std,
            nested_pipeline_depth: 0,
            lets: vec![],
            funcs: vec![],
            structs: vec![],
            enums: vec![],
            aliases: vec![],
            asserts: vec![],
            uses: vec![],
            modules: vec![],
            type_assertions: vec![],
            associated_items: vec![],
            trivial_lets: HashMap::new(),
            closures: HashMap::new(),
            lang_items: HashMap::new(),
            polys: HashMap::new(),
            poly_impls: vec![],
            errors: parse_session.errors.clone(),
            warnings: parse_session.warnings.clone(),
        }
    }

    // TODO: return visibility
    pub fn iter_item_names(&self) -> impl Iterator<Item = (InternedString, Span, NameKind)> {
        self.lets.iter().map(
            |r#let| (r#let.name, r#let.name_span, NameKind::Let { is_top_level: r#let.origin == LetOrigin::TopLevel })
        ).chain(
            self.funcs.iter().map(
                |func| (func.name, func.name_span, NameKind::Func)
            )
        ).chain(
            self.structs.iter().map(
                |r#struct| (r#struct.name, r#struct.name_span, NameKind::Struct)
            )
        )
        .chain(
            self.enums.iter().map(
                |r#enum| (r#enum.name, r#enum.name_span, NameKind::Enum)
            )
        )
        .chain(
            self.aliases.iter().map(
                |alias| (alias.name, alias.name_span, NameKind::Alias)
            )
        ).chain(
            self.uses.iter().map(
                |r#use| (r#use.name, r#use.name_span, NameKind::Use)
            )
        ).chain(
            self.modules.iter().map(
                |module| (module.name, module.name_span, NameKind::Use)
            )
        )
    }

    pub fn is_at_top_level_block(&self) -> bool {
        self.block_stack.len() == 1  // 1 is the top-level block's session
    }

    pub fn push_func_default_value(&mut self, default_value: Let) {
        self.block_stack.last_mut().unwrap().func_default_values.push(default_value);
    }

    pub fn push_lambda(&mut self, lambda: Func) {
        self.block_stack.last_mut().unwrap().lambdas.push(lambda);
    }

    // If a function has 5 params and 3 are unused, it throws 1 warning instead of 3.
    // If you want to throw multiple times, call this function multiple times with each name.
    pub fn warn_unused_names(&mut self, names: &HashMap<InternedString, (Span, NameKind, UseCount)>) {
        let mut names_by_kind: HashMap<(NameKind, bool), Vec<(InternedString, Span)>> = HashMap::new();

        for (name, (span, kind, count)) in names.iter() {
            if ((!self.is_in_debug_context && count.always == Counter::Never) || (self.is_in_debug_context && count.debug_only == Counter::Never)) && !name.eq(b"_") {
                let debug_only = count.debug_only != Counter::Never;
                match names_by_kind.entry((*kind, debug_only)) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().push((*name, *span));
                    },
                    Entry::Vacant(e) => {
                        e.insert(vec![(*name, *span)]);
                    },
                }
            }
        }

        for ((kind, debug_only), mut names) in names_by_kind.into_iter() {
            let note = if debug_only {
                Some(format!(
                    "{} only used in debug mode.",
                    if names.len() == 1 { "It is" } else { "These are" },
                ))
            } else {
                None
            };
            names.sort_by_key(|(_, span)| *span);
            self.warnings.push(Warning {
                kind: WarningKind::UnusedNames {
                    names: names.iter().map(
                        |(name, _)| *name
                    ).collect(),
                    kind,
                },
                spans: names.iter().map(
                    |(_, span)| RenderableSpan {
                        span: *span,
                        auxiliary: false,
                        note: None,
                    }
                ).collect(),
                note,
            });
        }
    }
}
