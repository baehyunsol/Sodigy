use crate::builtins::BuiltIns;
use crate::ctxt::TypeCkCtxt;
use sdg_ast::{Expr, ExprKind, LocalParseSession, Span, TailCall, TypeError, ValueKind};
use sdg_inter_mod::InterModuleContext;

/// If there's no type error, it returns Ok(ty).\
/// If there's an error, it adds the error to `session`, and continue.\
/// It continues until it cannot proceed, so that it can find as many errors as possible.
pub fn type_check(expr: &Expr, session: &mut LocalParseSession, funcs: &InterModuleContext, ctxt: &mut TypeCkCtxt) -> Result<Expr, ()> {
    match &expr.kind {
        ExprKind::Value(v) => match v {
            // TODO: we need some kind of context to know the type
            ValueKind::Identifier(name, origin) => Ok(ctxt.get_type_of_identifier(*name, *origin)),

            ValueKind::Object(id) => match funcs.search_by_id(*id) {
                Some(func) => todo!(),
                None => unreachable!(
                    "Internal Compiler Error 077A8CA855E"
                ),
            },

            ValueKind::Integer(_) => Ok(Expr::new_object(sdg_uid::prelude::int(), Span::dummy())),
            ValueKind::Real(_) => Ok(Expr::new_object(sdg_uid::prelude::number(), Span::dummy())),
            ValueKind::String(_) => Ok(Expr::new_object(sdg_uid::prelude::string(), Span::dummy())),
            ValueKind::Char(_) => Ok(Expr::new_object(sdg_uid::prelude::char(), Span::dummy())),
            ValueKind::Bytes(_) => Ok(Expr::new_object(sdg_uid::prelude::bytes(), Span::dummy())),

            ValueKind::Format(elements) => {
                // TODO: check that all the elements implement `to_string`
                for element in elements.iter() {
                    type_check(element, session, funcs, ctxt)?;
                }

                Ok(Expr::new_object(sdg_uid::prelude::string(), Span::dummy()))
            },

            ValueKind::List(elements) => if elements.is_empty() {
                // What then? toss it to the type-infer engine?
                // how about `List(Any)`, where `Any` is subtype of every type?
                todo!()
            } else {
                let mut elem_type = type_check(&elements[0], session, funcs, ctxt)?;

                for element in elements[1..].iter() {
                    let curr_elem_type = type_check(element, session, funcs, ctxt)?;

                    if elem_type.is_subtype_of(&curr_elem_type) {
                        elem_type = curr_elem_type;
                    } else if !curr_elem_type.is_subtype_of(&elem_type) {
                        // TODO: type error
                        // either of them is a subtype of the other
                    }

                }

                // `List(T)`
                Ok(Expr::new_call(
                    Expr::new_object(sdg_uid::prelude::list(), Span::dummy()),
                    vec![elem_type],
                    TailCall::NoTail,
                    Span::dummy(),
                ))
            },
            ValueKind::Tuple(elements) => {
                let mut types = Vec::with_capacity(elements.len());

                for element in elements.iter() {
                    types.push(type_check(element, session, funcs, ctxt)?);
                }

                Ok(Expr::new_tuple(types, Span::dummy()))
            },
            // should be removed by `name_resolve`
            ValueKind::Lambda(_, _) => unreachable!("Internal Compiler Error F77D0C6DE23"),
            ValueKind::Closure(_, _) => todo!(),
            ValueKind::Block { defs, value, id } => {
                for block_def in defs.iter() {
                    ctxt.register_block_defs(block_def, *id, session, funcs)?;
                }

                let result = type_check(value, session, funcs, ctxt);
                ctxt.drop_block_defs(*id);

                result
            },
        },
        ExprKind::Branch(cond, t, f) => {
            if let Ok(cond_type) = type_check(&cond, session, funcs, ctxt) {
                // TODO: if `cond_type` is not `prelude.Bool`, return error
            }

            if let (Ok(true_expr_type), Ok(false_expr_type)) = (type_check(&t, session, funcs, ctxt), type_check(&f, session, funcs, ctxt)) {
                // TODO: if `true_expr_type` is subtype of `false_expr_type` or vice versa,
                // return the smaller type, return error otherwise
            }

            // if there's no error, it must be returned above
            Err(())
        },
        ExprKind::Prefix(_, _) => todo!(),
        ExprKind::Infix(_, _, _) => todo!(),
        ExprKind::Postfix(_, _) => todo!(),
        ExprKind::Match(_, _, _) => todo!(),
        ExprKind::Call(func, args, _) => {
            let func_type = type_check(&func, session, funcs, ctxt)?;

            if !is_callable(&func_type) {
                session.add_error(TypeError::not_callable(
                    func.span,

                    // TODO: is it infallible?
                    func_type.to_string().to_rust_string().expect(
                        "Internal Compiler Error 623C7AF2D23"
                    ),
                ));
                return Err(());
            }

            let arg_types = get_arg_types(&func_type);
            let mut given_arg_types = Vec::with_capacity(args.len());

            for arg in args.iter() {
                given_arg_types.push((type_check(arg, session, funcs, ctxt)?, arg.span));
            }

            check_func_arg_types(arg_types, given_arg_types, session, func.span)?;
            let return_type = get_return_type(&func_type);

            Ok(return_type.clone())
        },
    }
}

fn is_callable(f: &Expr) -> bool {
    todo!()
}

// it's guaranteed that `f` is callable
fn get_arg_types(f: &Expr) -> Vec<&Expr> {
    todo!()
}

// it's guaranteed that `f` is callable
fn get_return_type(f: &Expr) -> &Expr {
    todo!()
}

// if there's an error, it adds the error directly to the session, not returning the error
fn check_func_arg_types(
    def_type: Vec<&Expr>,
    given_type: Vec<(Expr, Span)>,
    session: &mut LocalParseSession,
    func_span: Span,
) -> Result<(), ()> {

    if def_type.len() != given_type.len() {

        // it assumes that there's one unexpected arg
        // if more than 2 types are wrong, it just returns `WrongNumberArg`
        if def_type.len() + 1 == given_type.len() {
            todo!()
        }

        // it assumes that there's one missing arg
        // if more than 2 types are wrong, it just returns `WrongNumberArg`
        else if def_type.len() == given_type.len() + 1 {
            todo!()
        }

        else {
            session.add_error(
                TypeError::wrong_number_of_arg(func_span, def_type.len(), given_type.len())
            );
        }

        Err(())
    } else {
        let mut has_type_error = false;

        for index in 0..def_type.len() {
            if !given_type[index].0.is_subtype_of(&def_type[index]) {
                session.add_error(
                    TypeError::wrong_func_arg(
                        given_type[index].1,
                        todo!(),
                        todo!(),
                    )
                );
                has_type_error = true;
            }
        }

        if has_type_error {
            Err(())
        } else {
            Ok(())
        }
    }

}
