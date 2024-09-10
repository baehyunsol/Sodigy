use super::{Arg, Func, FuncKind};
use crate::{lower_ast_expr, lower_ast_ty};
use crate::attr::lower_ast_attributes;
use crate::error::HirError;
use crate::expr::try_warn_unnecessary_paren;
use crate::names::{IdentWithOrigin, NameBindingType, NameSpace};
use crate::session::HirSession;
use crate::warn::HirWarning;
use sodigy_ast as ast;
use sodigy_error::SodigyError;
use sodigy_intern::InternedString;
use sodigy_parse::IdentWithSpan;
use sodigy_session::SodigySession;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;
use std::collections::{HashMap, HashSet};

pub fn lower_ast_func(
    name: &IdentWithSpan,
    generics: &Vec<ast::GenericDef>,
    args: Option<&Vec<ast::ArgDef>>,
    return_val: &ast::Expr,
    return_ty: &Option<ast::TypeDef>,
    uid: Uid,
    session: &mut HirSession,
    used_names: &mut HashSet<IdentWithOrigin>,
    imports: &HashMap<InternedString, (SpanRange, Vec<IdentWithSpan>)>,
    attributes: &Vec<ast::Attribute>,
    name_space: &mut NameSpace,
) -> Result<Func, ()> {
    let mut hir_args = None;
    let mut has_error = false;

    name_space.enter_new_func_def();

    // don't let exprs access to func args until they're ready
    name_space.lock_func_args();

    for generic in generics.iter() {
        if let Err([name1, name2]) = name_space.push_generic(generic) {
            session.push_error(HirError::name_collision(name1, name2));
        }
    }

    if let Some(args) = args {
        let mut args_buffer = Vec::with_capacity(args.len());

        for arg in args.iter() {
            if let Err([name1, name2]) = name_space.push_arg(arg) {
                session.push_error(HirError::name_collision(name1, name2));
            }
        }

        for ast::ArgDef { name, ty, has_question_mark, attributes } in args.iter() {
            // lower ast::ArgDef to hir::Arg
            let ty = if let Some(ty) = ty {
                if let Ok(ty) = lower_ast_ty(
                    ty,
                    session,
                    used_names,
                    imports,
                    name_space,
                ) {
                    Some(ty)
                }

                else {
                    has_error = true;

                    None
                }
            }

            else {
                None
            };

            let attributes = if let Ok(attributes) = lower_ast_attributes(
                attributes,
                session,
                used_names,
                imports,
                name_space,
            ) {
                attributes
            } else {
                has_error = true;

                vec![]
            };

            args_buffer.push(Arg {
                name: *name,
                ty,
                has_question_mark: *has_question_mark,
                attributes,
            });
        }

        hir_args = Some(args_buffer);
    }

    if let Err([name1, name2]) = name_space.find_arg_generic_name_collision() {
        session.push_error(
            HirError::name_collision(name1, name2).set_message(
                String::from("Generic parameters and function arguments are in the same namespace. You cannot use the same names.")
            ).to_owned()
        );
    }

    name_space.unlock_func_args();

    try_warn_unnecessary_paren(return_val, session);

    let return_val = lower_ast_expr(
        return_val,
        session,
        used_names,
        imports,
        name_space,
    );

    let return_ty = return_ty.as_ref().map(
        |ty| lower_ast_ty(
            ty,
            session,
            used_names,
            imports,
            name_space,
        )
    );

    let attributes = lower_ast_attributes(
        attributes,
        session,
        used_names,
        imports,
        name_space,
    );

    // find unused names

    for (arg, name_origin) in name_space.iter_func_args() {
        if !used_names.contains(&IdentWithOrigin::new(arg.id(), name_origin)) {
            session.push_warning(HirWarning::unused_name(arg, NameBindingType::FuncArg));
        }
    }

    for (generic, name_origin) in name_space.iter_func_generics() {
        if !used_names.contains(&IdentWithOrigin::new(generic.id(), name_origin)) {
            session.push_warning(HirWarning::unused_name(generic, NameBindingType::FuncGeneric));
        }
    }

    name_space.leave_func_def();

    if has_error {
        return Err(());
    }

    Ok(Func {
        name: *name,
        generics: generics.clone(),
        args: hir_args,
        return_val: return_val?,
        return_ty: if let Some(ty) = return_ty { Some(ty?) } else { None },
        kind: FuncKind::Normal,
        attributes: attributes?,
        uid: uid,
    })
}
