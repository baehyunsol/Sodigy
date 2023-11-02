use super::{Arg, Func, FuncDeco};
use crate::{lower_ast_expr, lower_ast_ty};
use crate::err::HirError;
use crate::names::{IdentWithOrigin, NameBindingType, NameSpace};
use crate::session::HirSession;
use crate::warn::HirWarning;
use sodigy_ast as ast;
use sodigy_err::SodigyError;
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;
use std::collections::{HashMap, HashSet};

pub fn lower_ast_func(
    f: &ast::FuncDef,
    session: &mut HirSession,
    used_names: &mut HashSet<IdentWithOrigin>,
    use_cases: &HashMap<InternedString, (SpanRange, Vec<InternedString>)>,
    decorators: &Vec<ast::Decorator>,
    doc: Option<InternedString>,
    name_space: &mut NameSpace,
) -> Result<Func, ()> {
    let mut hir_args = None;
    let mut arg_lower_failure = false;

    name_space.enter_new_func_def();

    // don't let exprs access to func args until they're ready
    name_space.lock_func_args();

    for generic in f.generics.iter() {
        if let Err([name1, name2]) = name_space.push_generic(generic) {
            session.push_error(HirError::name_collision(name1, name2));
        }
    }

    if let Some(args) = &f.args {
        let mut args_buf = Vec::with_capacity(args.len());

        for arg in args.iter() {
            if let Err([name1, name2]) = name_space.push_arg(arg) {
                session.push_error(HirError::name_collision(name1, name2));
            }
        }

        for arg in args.iter() {
            // lower ast::ArgDef to hir::Arg
            let ty = if let Some(ty) = &arg.ty {
                if let Ok(ty) = lower_ast_ty(
                    &ty,
                    session,
                    used_names,
                    use_cases,
                    name_space,
                ) {
                    Some(ty)
                }

                else {
                    arg_lower_failure = true;

                    None
                }
            }

            else {
                None
            };

            args_buf.push(Arg {
                name: arg.name,
                ty,
                has_question_mark: arg.has_question_mark,
            });
        }

        hir_args = Some(args_buf);
    }

    if let Err([name1, name2]) = name_space.find_arg_generic_name_collision() {
        session.push_error(
            HirError::name_collision(name1, name2).set_message(
                String::from("Generic parameters and function arguments are in the same namespace. You cannot use the same names.")
            ).to_owned()
        );
    }

    name_space.unlock_func_args();

    let ret_val = lower_ast_expr(
        &f.ret_val,
        session,
        used_names,
        use_cases,
        name_space,
    );

    let ret_ty = f.ret_type.as_ref().map(
        |ty| lower_ast_ty(
            ty,
            session,
            used_names,
            use_cases,
            name_space,
        )
    );

    // find unused names

    for (arg, name_origin) in name_space.iter_func_args() {
        if !used_names.contains(&IdentWithOrigin::new(*arg.id(), name_origin)) {
            session.push_warning(HirWarning::unused_name(arg, NameBindingType::FuncArg));
        }
    }

    for (generic, name_origin) in name_space.iter_func_generics() {
        if !used_names.contains(&IdentWithOrigin::new(*generic.id(), name_origin)) {
            session.push_warning(HirWarning::unused_name(generic, NameBindingType::FuncGeneric));
        }
    }

    let decorators = lower_ast_func_decorators(
        decorators,
        session,
    );

    name_space.leave_func_def();

    if arg_lower_failure {
        return Err(());
    }

    Ok(Func {
        name: f.name,
        generics: f.generics.clone(),
        args: hir_args,
        ret_val: ret_val?,
        ret_ty: if let Some(ty) = ret_ty { Some(ty?) } else { None },
        decorators: decorators?,
        doc,
        uid: f.uid,
    })
}

pub fn lower_ast_func_decorators(
    decorators: &Vec<ast::Decorator>,
    session: &mut HirSession,
) -> Result<FuncDeco, ()> {
    let mut result = FuncDeco::default();

    for deco in decorators.iter() {
        todo!()
    }

    Ok(result)
}
