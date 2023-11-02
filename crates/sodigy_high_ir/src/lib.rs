use crate as hir;
use sodigy_ast::{self as ast, IdentWithSpan, StmtKind};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;
use std::collections::{HashMap, HashSet};

mod err;
mod expr;
mod func;
mod names;
mod pattern;
mod session;
mod warn;

use err::HirError;
pub use expr::Expr;
use expr::lower_ast_expr;
use func::lower_ast_func;
use names::{IdentWithOrigin, NameSpace};
pub use session::HirSession;
use warn::HirWarning;

pub fn lower_stmts(
    stmts: &Vec<ast::Stmt>,
    session: &mut HirSession
) -> Result<(), ()> {
    let mut curr_doc_comments = vec![];
    let mut curr_decorators = vec![];

    // only for warnings
    let preludes = session.get_prelude_names();

    // it's only for name-collision checking
    let mut names: HashMap<InternedString, IdentWithSpan> = HashMap::new();

    // `use x.y.z as z;` -> use_cases['z'] = ['x', 'y', 'z']
    let mut use_cases: HashMap<InternedString, (SpanRange, Vec<InternedString>)> = HashMap::new();

    // It's used to generate unused_name warnings
    let mut used_names: HashSet<IdentWithOrigin> = HashSet::new();

    // first iteration:
    // collect names from definitions and check name collisions
    // unfold all the `use`s: convert them into basic forms (`use x.y.z as z;`)
    for stmt in stmts.iter() {
        match &stmt.kind {
            StmtKind::Decorator(_)
            | StmtKind::DocComment(_) => { /* nop */ },
            StmtKind::Use(u) => {
                let mut aliases = vec![];
                u.unfold_alias(&mut aliases);

                for (from, to) in aliases.iter() {
                    if let Some(collision) = names.insert(*from.id(), *from) {
                        session.push_error(HirError::name_collision(*from, collision));
                    }

                    use_cases.insert(*from.id(), (*from.span(), to.to_vec()));
                }
            },
            stmt_kind => {
                let id = stmt_kind.get_id().unwrap();

                if let Some(collision) = names.insert(*id.id(), *id) {
                    session.push_error(HirError::name_collision(*id, collision));
                }
            },
        }
    }

    for id in names.values() {
        if preludes.contains(id.id()) {
            session.push_warning(HirWarning::redef_prelude(*id));
        }
    }

    // TODO: init name_space using the collected names
    let mut name_space = NameSpace::new();

    // second iteration
    // collect doc comments and decorators and find where they belong to
    // lower all the AST exprs to HIR exprs
    // convert enums and structs to defs
    for stmt in stmts.iter() {
        let span = stmt.span;

        match &stmt.kind {
            StmtKind::DocComment(c) => {
                curr_doc_comments.push((*c, span));
            },
            StmtKind::Decorator(d) => {
                curr_decorators.push(d.clone());
            },
            StmtKind::Func(f) => {
                // TODO: what do we do with it?
                lower_ast_func(
                    f,
                    session,
                    &mut used_names,
                    &use_cases,
                    &vec![],  // TODO: collect decorators
                    concat_doc_comments(&mut curr_doc_comments),
                    &mut name_space,
                );
            },
            _ => {
                // TODO
            },
        }
    }

    session.err_if_has_err()
}

pub fn lower_ast_ty(
    ty: &ast::TypeDef,
    session: &mut HirSession,
    used_names: &mut HashSet<IdentWithOrigin>,
    use_cases: &HashMap<InternedString, (SpanRange, Vec<InternedString>)>,
    name_space: &mut NameSpace,
) -> Result<Type, ()> {
    Ok(Type(lower_ast_expr(
        &ty.0,
        session,
        used_names,
        use_cases,
        name_space,
    )?))
}

fn concat_doc_comments(docs: &mut Vec<(InternedString, SpanRange)>) -> Option<InternedString> {
    if docs.is_empty() {
        None
    }

    else if docs.len() == 1 {
        Some(docs[0].0)
    }

    else {
        todo!()
    }
}

// TODO: independent module for these
pub struct Type(hir::Expr);
