use crate::{
    Assert,
    Expr,
    Func,
    FuncOrigin,
    Let,
    Session,
    Struct,
};
use sodigy_error::{Warning, WarningKind};
use sodigy_name_analysis::{
    Counter,
    NameKind,
    Namespace,
    UseCount,
};
use sodigy_number::InternedNumber;
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Block {
    pub group_span: Span,
    pub lets: Vec<Let>,
    pub value: Box<Expr>,
    pub asserts: Vec<Assert>,

    // It only counts names in `lets`.
    // It's later used for optimization.
    pub use_counts: HashMap<InternedString, UseCount>,
}

impl Block {
    pub fn from_ast(
        ast_block: &ast::Block,
        session: &mut Session,
        is_top_level: bool,
    ) -> Result<Block, ()> {
        let mut has_error = false;
        let mut lets = vec![];
        let mut asserts = vec![];

        session.func_default_values.push(vec![]);
        session.name_stack.push(Namespace::Block {
            names: ast_block.iter_names(is_top_level).map(
                |(k, v1, v2)| (k, (v1, v2, UseCount::new()))
            ).collect(),
        });

        for assert in ast_block.asserts.iter() {
            match Assert::from_ast(assert, session) {
                Ok(assert) => {
                    asserts.push(assert);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        for r#let in ast_block.lets.iter() {
            match Let::from_ast(r#let, session, is_top_level) {
                Ok(l) => {
                    lets.push(l);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        let func_origin = if is_top_level {
            FuncOrigin::TopLevel
        } else {
            FuncOrigin::Inline
        };

        // All the function declarations are stored in the top-level block.
        for func in ast_block.funcs.iter() {
            match Func::from_ast(func, session, func_origin, is_top_level) {
                Ok(f) => {
                    session.funcs.push(f);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        // All the struct declarations are stored in the top-level block.
        for r#struct in ast_block.structs.iter() {
            match Struct::from_ast(r#struct, session, is_top_level) {
                Ok(s) => {
                    session.structs.push(s);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        // If `ast_block.value` is None, that means the block is top-level.
        // An ast_block can be top-level or inline, but an hir_block is always an inline block.
        // If it's a top-level block, `HirSession::lower` will do proper handlings, so this function doesn't have to worry about anything.
        let value = match ast_block.value.as_ref().as_ref().map(|value| Expr::from_ast(&value, session)) {
            Some(Ok(value)) => Some(value),
            Some(Err(())) => {
                has_error = true;
                None
            },
            // If `ast_block.value` is None, it's a top-level block.
            // AST creates a `Block` instance for the top-level block, but HIR doesn't.
            // So we first use a dummy value. The HIR session will do the cleanup.
            None => Some(Expr::Number {
                n: InternedNumber::from_u32(0, true),
                span: Span::None,
            }),
        };

        let mut use_counts = HashMap::new();
        let Some(Namespace::Block { names }) = session.name_stack.pop() else { unreachable!() };

        // TODO:
        //    inline-block: always warn unused names
        //    top-level-block: only warn unused `use`s
        //    how about debug-only names in top-level?
        for (name, (span, kind, count)) in names.iter() {
            if let NameKind::Let { .. } = kind {
                use_counts.insert(*name, *count);
            }

            if (!session.is_in_debug_context && count.always == Counter::Never) ||
                (session.is_in_debug_context && count.debug_only == Counter::Never) {
                let mut extra_message = None;

                if count.debug_only != Counter::Never {
                    extra_message = Some(String::from("This value is only used in debug mode."));
                }

                session.warnings.push(Warning {
                    kind: WarningKind::UnusedName {
                        name: *name,
                        kind: *kind,
                    },
                    span: *span,
                    extra_message,
                    ..Warning::default()
                });
            }
        }

        for func_default_value in session.func_default_values.pop().unwrap() {
            lets.push(func_default_value);
        }

        if has_error {
            Err(())
        }

        else {
            Ok(Block {
                group_span: ast_block.group_span,
                lets,
                asserts,
                value: Box::new(value.unwrap()),
                use_counts,
            })
        }
    }
}
