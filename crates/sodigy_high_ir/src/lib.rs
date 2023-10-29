use sodigy_ast::{self as ast, IdentWithSpan, StmtKind};
use sodigy_intern::InternedString;
use std::collections::HashMap;

mod err;
mod expr;
mod names;
mod session;

use err::HirError;
pub use session::HirSession;

pub fn from_stmts(
    stmts: &Vec<ast::Stmt>,
    session: &mut HirSession
) -> Result<(), ()> {
    let mut curr_doc_comments = vec![];
    let mut curr_decorators = vec![];
    let mut module_defs = HashMap::new();

    // it's only for name-collision checking
    let mut names: HashMap<InternedString, IdentWithSpan> = HashMap::new();

    // `use x.y.z as z;` -> use_cases['z'] = ['x', 'y', 'z']
    let mut use_cases: HashMap<IdentWithSpan, Vec<InternedString>> = HashMap::new();

    // first iteration:
    // collect names from definitions and check name collisions
    // unfold all the `use`s: convert them into basic forms (`use x.y.z as z;`)
    for stmt in stmts.iter() {
        match &stmt.kind {
            StmtKind::Decorator(_)
            | StmtKind::DocComment(_) => { /* nop */ },
            StmtKind::Use(u) => {
                for (from, to) in u.unfold_alias().iter() {
                    if let Some(collision) = names.insert(*from.id(), *from) {
                        session.push_error(HirError::name_collision(*from, collision));
                        return Err(());
                    }

                    use_cases.insert(*from, to.to_vec());
                }
            },
            stmt_kind => {
                let id = stmt_kind.get_id().unwrap();

                if let Some(collision) = names.insert(*id.id(), *id) {
                    session.push_error(HirError::name_collision(*id, collision));
                    return Err(());
                }
            },
        }
    }

    // second iteration
    // collect doc comments and decorators and find where they belong to
    // lower all the AST exprs to HIR exprs
    // convert enums and structs to defs
    for stmt in stmts.iter() {
        let span = stmt.span;

        match &stmt.kind {
            StmtKind::DocComment(c) => {
                curr_doc_comments.push((c.to_string(), span));
            },
            StmtKind::Decorator(d) => {
                curr_decorators.push(d.clone());
            },
            StmtKind::Module(m) => {
                // TODO: merge doc_comments
                // TODO: merge decorators
                // TODO: set modules's doc and decorators
                module_defs.insert(*m.id(), m.clone());
            },
            _ => {
                // TODO
            },
        }
    }

    // TODO
    Ok(())
}
