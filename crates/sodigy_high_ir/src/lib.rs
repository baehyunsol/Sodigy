use sodigy_ast::{Stmt, StmtKind};
use std::collections::HashMap;

// from Vec<Stmt>

// merge doc comments
// find where doc comments and decorators belong to
// find all the origins of identifiers
// convert enums and structs into funcs
// find name collisions
// resolve uses (e.g. `a` -> `b.c.a` if `use b.c.a;`)
// distinguish lambdas and closures

pub fn from_stmts(stmts: &mut Vec<Stmt>) {
    let mut curr_doc_comments = vec![];
    let mut curr_decorators = vec![];
    let mut module_defs = HashMap::new();

    for stmt in stmts.iter_mut() {
        let span = stmt.span;

        match &mut stmt.kind {
            StmtKind::DocComment(c) => {
                curr_doc_comments.push((c.to_string(), span));
            },
            StmtKind::Decorator(d) => {
                curr_decorators.push(d.clone());
            },
            StmtKind::Module(m) => {
                // TODO: check name collision

                // TODO: merge doc_comments
                // TODO: merge decorators
                // TODO: set modules's doc and decorators
                module_defs.insert(*m.id(), m.clone());
            },
            _ => todo!(),
        }
    }
}
