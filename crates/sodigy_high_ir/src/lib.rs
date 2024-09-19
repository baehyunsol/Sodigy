#![deny(unused_imports)]

use crate as hir;
use log::info;
use sodigy_ast::{self as ast, LetKind, StmtKind};
use sodigy_attribute::Attribute;
use sodigy_error::SodigyError;
use sodigy_intern::InternedString;
use sodigy_parse::IdentWithSpan;
use sodigy_session::SodigySession;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;
use std::collections::{HashMap, HashSet};

mod attr;
mod doc_comment;
mod endec;
mod enum_;
mod error;
pub mod expr;
mod fmt;
mod func;
mod module;
mod names;
mod pattern;
mod scope;
mod session;
mod struct_;
mod walker;
mod warn;

pub use attr::lower_ast_attributes;
use doc_comment::concat_doc_comments;
use enum_::lower_ast_enum;
use error::HirError;
pub use expr::{
    Branch, BranchArm,
    Expr, ExprKind,
    StructInit,
    StructInitField,
};
use expr::{
    lower_ast_expr,
    try_warn_unnecessary_paren,
    lambda::{
        give_names_to_lambdas,
        try_convert_closures_to_lambdas,
        LambdaCollectCtxt,
    },
};
pub use func::{Arg, Func, FuncKind};
use func::lower_ast_func;
pub use names::NameOrigin;
pub use module::Module;
use names::{IdentWithOrigin, NameSpace};
pub use names::NameBindingType;
pub use scope::{Scope, ScopedLet};
pub use session::HirSession;
use struct_::lower_ast_struct;
pub use walker::{
    EmptyWalkerState,
    mut_walker_expr,
    mut_walker_func,
    walker_expr,
    walker_func,
};
use warn::HirWarning;

pub fn lower_stmts(
    stmts: &Vec<ast::Stmt>,
    session: &mut HirSession,
) -> Result<(), ()> {
    info!("sodigy_high_ir::lower_stmts()");

    let mut ast_attributes = vec![];

    // only for warnings
    let preludes = session.get_prelude_names();

    // it collects names and uids of items in this module
    let mut names: HashMap<InternedString, (IdentWithSpan, Option<Uid>)> = HashMap::new();

    // `import x.y.z as z;` -> imports['z'] = ['x', 'y', 'z']
    // span is of `z`, it's for error messages
    let mut imports: HashMap<InternedString, (SpanRange, Vec<IdentWithSpan>)> = HashMap::new();

    // It's used to generate unused_name warnings
    let mut used_names: HashSet<IdentWithOrigin> = HashSet::new();

    // first iteration:
    // collect names from definitions and check name collisions
    // unfold all the `import`s: convert them into basic forms (`import x.y.z as z;`)
    for stmt in stmts.iter() {
        match &stmt.kind {
            StmtKind::Decorator(_)
            | StmtKind::DocComment(_) => { /* nop */ },
            StmtKind::Import(imp) => {
                let mut aliases = vec![];
                imp.unfold_alias(&mut aliases);

                for (from, to) in aliases.iter() {
                    if let Some((collision, _)) = names.insert(from.id(), (*from, None)) {
                        session.push_error(HirError::name_collision(*from, collision));
                    }

                    imports.insert(from.id(), (*from.span(), to.to_vec()));
                }
            },
            StmtKind::Module(name, uid) => {
                // `module A;` implies `import A;`
                if let Some((collision, _)) = names.insert(name.id(), (*name, Some(*uid))) {
                    let mut error = HirError::name_collision(*name, collision);

                    // TODO: it only works if `import foo;` comes before `module foo;`
                    if imports.contains_key(&name.id()) {
                        error.set_message(
                            format!(
                                "`module {};` implies `import {};`. You don't have to import `{}` again.",
                                name.id(),
                                name.id(),
                                name.id(),
                            )
                        );
                    }

                    session.push_error(error);
                }

                imports.insert(name.id(), (*name.span(), vec![*name]));
            },
            stmt_kind => {
                let id = if let Some(id) = stmt_kind.get_id() {
                    id
                } else {
                    // it must be `let pattern`, which is not implemented yet
                    // it'll be handled later
                    continue;
                };

                let uid = stmt_kind.get_uid().unwrap();

                if let Some((collision, _)) = names.insert(id.id(), (id, Some(uid))) {
                    session.push_error(HirError::name_collision(id, collision));
                }
            },
        }
    }

    let mut imported_names_set = HashSet::new();

    // for `import x.y.z;`, the name `x` is an imported name: the session will
    // search for this name later
    for (_, names) in imports.values() {
        if !imported_names_set.contains(&names[0].id()) {
            imported_names_set.insert(names[0].id());
            session.imported_names.push(names[0]);
        }
    }

    for (id, _) in names.values() {
        if preludes.contains(&id.id()) {
            session.push_warning(HirWarning::redef_prelude(*id));
        }
    }

    let mut name_space = NameSpace::new();
    name_space.push_globals(&names);

    // second iteration
    // collect doc comments and decorators and find where they belong to
    // lower all the AST exprs to HIR exprs
    // convert enums and structs to defs
    for stmt in stmts.iter() {
        let span = stmt.span;

        match &stmt.kind {
            StmtKind::DocComment(c) => {
                ast_attributes.push(Attribute::DocComment(IdentWithSpan::new(*c, span)));
            },
            StmtKind::Decorator(d) => {
                ast_attributes.push(Attribute::Decorator(d.clone()));
            },
            StmtKind::Let(l) => {
                match &l.kind {
                    LetKind::Callable { .. }
                    | LetKind::Incallable { .. } => {
                        let f = match &l.kind {
                            LetKind::Callable {
                                name, generics,
                                args, return_value,
                                return_type, uid,
                            } => lower_ast_func(
                                name,
                                generics,
                                Some(args),
                                return_value,
                                return_type,
                                *uid,
                                session,
                                &mut used_names,
                                &imports,
                                &ast_attributes,
                                &mut name_space,
                            ),
                            LetKind::Incallable {
                                name, generics,
                                return_value, return_type, uid,
                            } => lower_ast_func(
                                name,
                                generics,
                                None,
                                return_value,
                                return_type,
                                *uid,
                                session,
                                &mut used_names,
                                &imports,
                                &ast_attributes,
                                &mut name_space,
                            ),
                            _ => unreachable!(),
                        };

                        let mut f = if let Ok(f) = f { f } else { continue; };
                        let mut lambda_context = LambdaCollectCtxt::new(session);

                        // TODO: `try_convert_closures_to_lambdas` on StructDefs and EnumDefs
                        try_convert_closures_to_lambdas(&mut f);
                        give_names_to_lambdas(&mut f, &mut lambda_context);

                        // TODO: `try_convert_closures_to_lambdas` on these
                        for func in lambda_context.collected_lambdas.into_iter() {
                            assert!(session.get_results_mut().insert(func.uid, func).is_none());
                        }

                        assert!(session.get_results_mut().insert(f.uid, f).is_none());
                    },
                    LetKind::Enum {
                        name,
                        generics,
                        variants,
                        uid,
                    } => {
                        // errors are pushed to `session` by this function
                        // there's no extra thing to do
                        let _ = lower_ast_enum(
                            name,
                            generics,
                            variants,
                            *uid,
                            session,
                            &mut used_names,
                            &imports,
                            &ast_attributes,
                            &mut name_space,
                        );
                    },
                    LetKind::Struct {
                        name,
                        generics,
                        fields,
                        uid,
                    } => {
                        // errors are pushed to `session` by this function
                        // there's no extra thing to do
                        let _ = lower_ast_struct(
                            name,
                            generics,
                            fields,
                            *uid,
                            session,
                            &mut used_names,
                            &imports,
                            &ast_attributes,
                            &mut name_space,
                            None,  // not an enum variant
                        );
                    },
                    LetKind::Pattern(pattern, _) => {
                        session.push_error(HirError::todo("top-level pattern destructure", pattern.span));
                    },
                }

                ast_attributes.clear();
            },
            StmtKind::Module(name, uid) => {
                let attributes = if let Ok(attributes) = lower_ast_attributes(
                    &ast_attributes,
                    session,
                    &mut used_names,
                    &imports,
                    &mut name_space,
                ) {
                    ast_attributes.clear();
                    attributes
                } else {
                    continue
                };

                session.modules.push(Module {
                    name: *name,
                    uid: *uid,
                    attributes,
                });
            },
            StmtKind::Import(_) => {
                // already handled
            },
        }
    }

    // order of the elements of `imports.iter()` is not consistent -> it has to be sorted
    session.sort_errors_and_warnings();

    // another issue with order here: it also has to be sorted
    session.imported_names.sort_by_key(|idws| *idws.span());

    session.err_if_has_error()
}

pub fn lower_ast_ty(
    ty: &ast::TypeDef,
    session: &mut HirSession,
    used_names: &mut HashSet<IdentWithOrigin>,
    imports: &HashMap<InternedString, (SpanRange, Vec<IdentWithSpan>)>,
    name_space: &mut NameSpace,
) -> Result<Type, ()> {
    try_warn_unnecessary_paren(ty.as_expr(), session);

    Ok(Type(lower_ast_expr(
        ty.as_expr(),
        session,
        used_names,
        imports,
        name_space,
    )?))
}

#[derive(Clone)]
pub struct Type(hir::Expr);

impl Type {
    pub fn as_expr(&self) -> &hir::Expr {
        &self.0
    }
}
