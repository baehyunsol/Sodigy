#![deny(unused_imports)]

use crate as hir;
use sodigy_ast::{self as ast, IdentWithSpan, LetKind, StmtKind};
use sodigy_intern::InternedString;
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
mod names;
mod pattern;
mod session;
mod struct_;
mod walker;
mod warn;

pub use attr::{Attribute, Decorator};
use doc_comment::concat_doc_comments;
use enum_::lower_ast_enum;
use error::HirError;
pub use expr::{
    Branch, BranchArm,
    Expr, ExprKind,
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
pub use func::Func;
use func::lower_ast_func;
pub use names::NameOrigin;
use names::{IdentWithOrigin, NameBindingType, NameSpace};
pub use session::HirSession;
use struct_::lower_ast_struct;
use warn::HirWarning;

pub fn lower_stmts(
    stmts: &Vec<ast::Stmt>,
    session: &mut HirSession
) -> Result<(), ()> {
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
            // TODO: collect attributes in AST level: AST::Let has `.attributes`
            StmtKind::DocComment(c) => {
                ast_attributes.push(ast::Attribute::DocComment(IdentWithSpan::new(*c, span)));
            },
            StmtKind::Decorator(d) => {
                ast_attributes.push(ast::Attribute::Decorator(d.clone()));
            },
            StmtKind::Let(l) => {
                match &l.kind {
                    LetKind::Callable { .. }
                    | LetKind::Incallable { .. } => {
                        let f = match &l.kind {
                            LetKind::Callable {
                                name, generics,
                                args, return_val,
                                return_ty, uid,
                            } => lower_ast_func(
                                name,
                                generics,
                                Some(args),
                                return_val,
                                return_ty,
                                *uid,
                                session,
                                &mut used_names,
                                &imports,
                                &ast_attributes,
                                &mut name_space,
                            ),
                            LetKind::Incallable {
                                name, generics,
                                return_val, return_ty, uid,
                            } => lower_ast_func(
                                name,
                                generics,
                                None,
                                return_val,
                                return_ty,
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
                            session.func_defs.insert(func.name.id(), func);
                        }

                        session.func_defs.insert(f.name.id(), f);
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
                        );
                    },
                    LetKind::Pattern(pattern, _) => {
                        session.push_error(HirError::todo("top-level pattern destructure", pattern.span));
                    },
                }

                ast_attributes.clear();
            },
            _ => {
                // TODO
            },
        }
    }

    for (name, (span, _)) in imports.iter() {
        if !used_names.contains(&IdentWithOrigin::new(*name, NameOrigin::Global { origin: None })) {
            session.push_warning(HirWarning::unused_name(
                IdentWithSpan::new(*name, *span),
                NameBindingType::Import,
            ));
        }
    }

    session.err_if_has_err()
}

pub fn lower_ast_ty(
    ty: &ast::TypeDef,
    session: &mut HirSession,
    used_names: &mut HashSet<IdentWithOrigin>,
    imports: &HashMap<InternedString, (SpanRange, Vec<IdentWithSpan>)>,
    name_space: &mut NameSpace,
) -> Result<Type, ()> {
    try_warn_unnecessary_paren(&ty.0, session);

    Ok(Type(lower_ast_expr(
        &ty.0,
        session,
        used_names,
        imports,
        name_space,
    )?))
}

#[derive(Clone)]
pub struct Type(hir::Expr);
