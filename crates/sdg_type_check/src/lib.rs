/*
use sdg_ast::{Expr, ExprKind, LocalParseSession, ValueKind};

/// If there's no type error, it returns Ok(ty).\
/// If there's an error, it adds error to `session`, and continue.\
/// It continues until it cannot proceed, so that it can find as many errors as possible.
fn type_check(expr: &Expr, session: &LocalParseSession) -> Result<Expr, ()> {
    match expr.kind {
        ExprKind::Value(v) => match v {
            // TODO: we need some kind of context to know the type
            ValueKind::Identifier(_, _) => todo!(),

            // TODO: represent prelude.Int
            ValueKind::Integer(_) => todo!(),

            // TODO: represent prelude.Number
            ValueKind::Real(_) => todo!(),

            ValueKind::List(elements) => if elements.is_empty() {
                // What then? toss it to the type-infer engine?
                // how about `List(Any)`, where `Any` is subtype of every type?
            } else {
                // TODO: check the type of all the elements, and then...
            },
        },
        ExprKind::Branch(cond, t, f) => {
            let cond_type = type_check(&cond, session)?;

            // TODO: if `cond_type` is not `prelude.Bool`, return error

            let true_expr_type = type_check(&t, session)?;
            let false_expr_type = type_check(&f, session)?;

            // TODO: if `true_expr_type` is subtype of `false_expr_type` or vice versa,
            // return the smaller type, return error otherwise
        },
        ExprKind::Call(func, args) => {
            let func_type = type_check(&func, session)?;

            // TODO: make sure that `func` is callable

            let arg_types = get_arg_types(&func_type);
            let return_type = get_return_type(&func_type);

            // TODO: make it more flexible, like rustc
            // e.g: if `(B, C, D)` is given when `(A, B, C, D)` is expected,
            // it's likely that the programmer missed an argument.
            // don't generate errors for `B`, `C`, and `D` in this case.

            if arg_types.len() != args.len() {
                // TODO: Err
            }

            for (index, arg_type) in arg_types.iter().enumerate() {
                // TODO: if args[index] is not subtype of arg_type, session.add_error
                // don't break though
            }

            // TODO: return `Ok(return_type)`
        },
    }

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
*/