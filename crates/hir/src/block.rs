use crate::{
    Alias,
    Assert,
    Expr,
    Func,
    FuncOrigin,
    Let,
    Module,
    Session,
    Struct,
    Use,
};
use sodigy_error::{Error, ErrorKind, Warning, WarningKind};
use sodigy_name_analysis::{
    Counter,
    NameKind,
    Namespace,
    UseCount,
};
use sodigy_number::InternedNumber;
use sodigy_parse as ast;
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct Block {
    pub group_span: Span,
    pub lets: Vec<Let>,
    pub asserts: Vec<Assert>,
    pub value: Box<Expr>,

    // TODO: The term `use` in `use_counts` is confusing.
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

        let mut let_cycle_check_vertices: HashSet<Span> = ast_block.lets.iter().map(
            |r#let| r#let.name_span
        ).collect();
        let mut let_cycle_check_edges: HashMap<Span, Vec<Span>> = HashMap::new();
        let alias_cycle_check_vertices: HashSet<Span> = ast_block.aliases.iter().map(
            |alias| alias.name_span
        ).collect();
        let mut alias_cycle_check_edges: HashMap<Span, Vec<Span>> = HashMap::new();

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
                Ok(r#let) => {
                    let_cycle_check_edges.insert(
                        r#let.name_span,
                        r#let.foreign_names.iter().filter(
                            |(_, (_, span))| let_cycle_check_vertices.contains(span)
                        ).map(
                            |(_, (_, span))| *span
                        ).collect(),
                    );
                    lets.push(r#let);
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
                Ok(func) => {
                    session.funcs.push(func);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        // All the struct declarations are stored in the top-level block.
        for r#struct in ast_block.structs.iter() {
            match Struct::from_ast(r#struct, session, is_top_level) {
                Ok(r#struct) => {
                    session.structs.push(r#struct);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        // All the aliases are stored in the top-level block.
        for alias in ast_block.aliases.iter() {
            match Alias::from_ast(alias, session) {
                Ok(alias) => {
                    alias_cycle_check_edges.insert(
                        alias.name_span,
                        alias.foreign_names.iter().filter(
                            |(_, (_, span))| alias_cycle_check_vertices.contains(span)
                        ).map(
                            |(_, (_, span))| *span
                        ).collect(),
                    );
                    session.aliases.push(alias);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        // All the uses are stored in the top-level block.
        for r#use in ast_block.uses.iter() {
            match Use::from_ast(r#use, session) {
                Ok(r#use) => {
                    session.uses.push(r#use.clone());
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        for module in ast_block.modules.iter() {
            match Module::from_ast(module, session) {
                Ok(module) => {
                    session.modules.push(module);
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
                let mut note = None;

                if count.debug_only != Counter::Never {
                    note = Some(String::from("This value is only used in debug mode."));
                }

                session.warnings.push(Warning {
                    kind: WarningKind::UnusedName {
                        name: *name,
                        kind: *kind,
                    },
                    spans: span.simple_error(),
                    note,
                });
            }
        }

        for func_default_value in session.func_default_values.pop().unwrap() {
            let_cycle_check_vertices.insert(func_default_value.name_span);
            let_cycle_check_edges.insert(
                func_default_value.name_span,
                func_default_value.foreign_names.iter().filter(
                    |(_, (_, span))| let_cycle_check_vertices.contains(span)
                ).map(
                    |(_, (_, span))| *span
                ).collect(),
            );
            lets.push(func_default_value);
        }

        // TOOD: It only underlines the definitions of the names.
        //       I want it to underline the actual uses of the names.
        if let Some(cycle) = find_cycle(
            let_cycle_check_vertices.into_iter().collect(),
            let_cycle_check_edges,
        ) {
            let span_to_name: HashMap<Span, InternedString> = lets.iter().map(
                |r#let| (r#let.name_span, r#let.name)
            ).collect();
            has_error = true;
            session.errors.push(Error {
                kind: ErrorKind::CyclicLet {
                    names: cycle.iter().map(|span| *span_to_name.get(span).unwrap()).collect(),
                },
                spans: cycle.iter().map(
                    |span| RenderableSpan {
                        span: *span,
                        auxiliary: false,
                        note: None,
                    }
                ).collect(),
                note: None,
            });
        }

        // TOOD: It only underlines the definitions of the names.
        //       I want it to underline the actual uses of the names.
        if let Some(cycle) = find_cycle(
            alias_cycle_check_vertices.into_iter().collect(),
            alias_cycle_check_edges,
        ) {
            let span_to_name: HashMap<Span, InternedString> = ast_block.aliases.iter().map(
                |alias| (alias.name_span, alias.name)
            ).collect();
            has_error = true;
            session.errors.push(Error {
                kind: ErrorKind::CyclicAlias {
                    names: cycle.iter().map(|span| *span_to_name.get(span).unwrap()).collect(),
                },
                spans: cycle.iter().map(
                    |span| RenderableSpan {
                        span: *span,
                        auxiliary: false,
                        note: None,
                    }
                ).collect(),
                note: None,
            });
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

// VIBE NOTE: this function's drafted by Perplexity (I'm not sure which model it was),
//            and modified by me.
// If it finds a cycle, it immediately exits. The result has all the vertices of the cycle.
fn find_cycle(
    vertices: Vec<Span>,
    edges: HashMap<Span, Vec<Span>>,
) -> Option<Vec<Span>> {
    fn dfs(
        node: Span,
        edges: &HashMap<Span, Vec<Span>>,
        visited: &mut HashSet<Span>,
        stack: &mut Vec<Span>,
        on_stack: &mut HashSet<Span>,
    ) -> Option<Vec<Span>> {
        visited.insert(node);
        stack.push(node);
        on_stack.insert(node);

        if let Some(neighbors) = edges.get(&node) {
            for neighbor in neighbors.iter() {
                if *neighbor == node {
                    return Some(vec![node]);  // self-cycle
                }

                if !visited.contains(neighbor) {
                    if let Some(cycle) = dfs(
                        *neighbor,
                        edges,
                        visited,
                        stack,
                        on_stack,
                    ) {
                        return Some(cycle);
                    }
                }

                else if on_stack.contains(neighbor) {
                    let index = stack.iter().rposition(|node| node == neighbor).unwrap();
                    return Some(stack[index..].to_vec());
                }
            }
        }

        on_stack.remove(&node);
        stack.pop().unwrap();
        None
    }

    let mut visited = HashSet::new();
    let mut stack = vec![];
    let mut on_stack = HashSet::new();

    for vertex in vertices.iter() {
        if !visited.contains(vertex) {
            if let Some(cycle) = dfs(*vertex, &edges, &mut visited, &mut stack, &mut on_stack) {
                return Some(cycle);
            }
        }
    }

    None
}
