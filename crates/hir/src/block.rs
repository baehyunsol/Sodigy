use crate::{
    Alias,
    Assert,
    Enum,
    Expr,
    Func,
    FuncOrigin,
    Let,
    Module,
    Session,
    Struct,
    Use,
};
use sodigy_error::{Error, ErrorKind};
use sodigy_name_analysis::{
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
    pub fn from_ast(ast_block: &ast::Block, session: &mut Session) -> Result<Block, ()> {
        let mut has_error = false;
        let mut lets = vec![];
        let mut asserts = vec![];

        // NOTE: You must do this before calling `.is_at_top_level_block()` because
        // the method counts the number of `BlockSession`s!
        session.block_stack.push(BlockSession::new());

        let mut let_cycle_check_vertices: HashSet<Span> = ast_block.lets.iter().map(
            |r#let| r#let.name_span
        ).collect();
        let mut let_cycle_check_edges: HashMap<Span, Vec<Span>> = HashMap::new();
        let alias_cycle_check_vertices: HashSet<Span> = ast_block.aliases.iter().map(
            |alias| alias.name_span
        ).collect();
        let mut alias_cycle_check_edges: HashMap<Span, Vec<Span>> = HashMap::new();

        session.name_stack.push(Namespace::Block {
            names: ast_block.iter_names(session.is_at_top_level_block()).map(
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
            match Let::from_ast(r#let, session) {
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

        let func_origin = if session.is_at_top_level_block() {
            FuncOrigin::TopLevel
        } else {
            FuncOrigin::Inline
        };

        // All the function declarations are stored in the top-level block.
        for func in ast_block.funcs.iter() {
            match Func::from_ast(func, session, func_origin) {
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
            match Struct::from_ast(r#struct, session) {
                Ok(r#struct) => {
                    session.structs.push(r#struct);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        for r#enum in ast_block.enums.iter() {
            match Enum::from_ast(r#enum, session) {
                Ok(r#enum) => {
                    session.enums.push(r#enum);
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
                    session.uses.push(r#use);
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

        // TODO: it has to warn unused names...
        //       but we need specs for visibility
        for (name, (_, kind, count)) in names.iter() {
            if let NameKind::Let { .. } = kind {
                use_counts.insert(*name, *count);
            }
        }

        // If it's top-level, mir will check unused names.
        // If it's from a pipeline, `Expr::from_ast` will throw an error if there's an unused name.
        if !session.is_at_top_level_block() && !ast_block.from_pipeline {
            session.warn_unused_names(&names);
        }

        let mut block_session = session.block_stack.pop().unwrap();

        for func_default_value in block_session.func_default_values.drain(..) {
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

        let lambdas: Vec<Func> = block_session.lambdas.drain(..).collect();

        for (func, closure_info) in session.check_captured_names(&mut lambdas, &lets) {
            session.funcs.push(func);
            // TODO: what do we do if it's a closure?
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

pub struct BlockSession {
    // Lambda (maybe closure) defined in this block
    pub lambdas: Vec<Func>,

    // Default values of functions in the current block. A default value is
    // lowered to a `let` statement.
    // When it leaves a block, it pops `let` statements and pushes them
    // to the current block.
    pub func_default_values: Vec<Let>,
}

impl BlockSession {
    pub fn new() -> Self {
        BlockSession {
            lambdas: vec![],
            func_default_values: vec![],
        }
    }
}

impl Session {
    pub fn check_captured_names(&self, lambdas: &mut Vec<Func>, lets: &[Let]) -> Vec<(Func, _)> {
        for lambda in lambdas.iter() {
            // In `fn(x) = \(y) => x + y;`, there's nothing we can do with `x`.
            // We have to capture `x` and make a closure.
            let mut names_to_capture = vec![];

            // `Int` in `\(x: Int) => x + 1` is a foreign name, but we don't have to capture it!
            let mut names_not_to_capture = vec![];

            // In `{ let x = 3; \(y) => x + y }`, if the compiler is smart enough,
            // we can rewrite the lambda to `\(y) => 3 + y`. We'll keep such cases
            // in this vector, and post-mir will do the optimization. We can't do
            // the optimization until inter-mir, because doing the substitution
            // will make type-errors (if exists) less readable.
            let mut names_maybe_not_to_capture = vec![];

            for (foreign_name, (origin, def_span)) in lambda.foreign_names.iter() {
                // NOTE: inter-hir guarantees that NameKind::Use are never local values
            }
        }

        todo!()
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
