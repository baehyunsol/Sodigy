use crate::builtins::BuiltIns;
use sdg_ast::{Expr, ExprKind, Span, TypeError, ValueKind};
use std::rc::Rc;

mod builtins;
mod ctxt;
mod typeck;

use ctxt::EvalCtxt;

pub use ctxt::TypeCkCtxt;
pub use typeck::type_check_ast;

// It has lots of `clone`s, that's due to the difference between
// Sodigy's immutable Rc-based system and Rust's ownership system.

pub fn evaluate(val: Rc<Expr>, context: &mut EvalCtxt) -> Result<Rc<Expr>, ()> {
    loop {
        match &val.kind {
            ExprKind::Value(v) => match v {
                ValueKind::Identifier(name, origin) => {
                    return Ok(Rc::new(context.evaluate_identifier(*name, *origin)?))
                },
                ValueKind::Integer(_)
                | ValueKind::Real(_)
                | ValueKind::String(_)
                | ValueKind::Char(_)
                | ValueKind::Bytes(_)
                | ValueKind::Object(_) => {
                    return Ok(val.clone());
                },
                ValueKind::Format(elements) => todo!(),
                ValueKind::List(elements)
                | ValueKind::Tuple(elements) => {
                    let mut evaluated_elements = Vec::with_capacity(elements.len());

                    for element in elements.iter() {
                        let element = evaluate(Rc::new(element.clone()), context)?;
                        evaluated_elements.push(element.as_ref().clone());
                    }

                    if v.is_tuple() {
                        return Ok(Rc::new(Expr::new_tuple(evaluated_elements, Span::dummy())));
                    } else {
                        return Ok(Rc::new(Expr::new_list(evaluated_elements, Span::dummy())));
                    }
                },
                // should be removed by `name_resolve`
                ValueKind::Lambda(_, _) => unreachable!("Internal Compiler Error 1426F1C713D"),
                ValueKind::Closure(name, captured_variables) => todo!(),
                ValueKind::Block { defs, value, .. } => todo!(),
            },
            ExprKind::Call(f, args, tail) => {
                let mut args_eval = Vec::with_capacity(args.len());

                for arg in args.iter() {
                    args_eval.push(evaluate(Rc::new(arg.clone()), context)?);
                }

                let func_eval = evaluate(Rc::new(f.as_ref().clone()), context)?;

                match func_eval.kind {
                    ExprKind::Value(ValueKind::Object(id)) => todo!(),

                    // TODO: dynamic function calls
                    _ => todo!(),
                }

                if tail.is_tail() {
                    context.set_args(args_eval);
                    // TODO
                } else {
                    // TODO
                }
            },
            ExprKind::Prefix(op, expr) => todo!(),
            ExprKind::Postfix(op, expr) => todo!(),
            ExprKind::Infix(op, expr1, expr2) => todo!(),
            ExprKind::Match(val, branches, _) => {
                let match_val = evaluate(Rc::new(val.as_ref().clone()), context)?;

                todo!()
            }
            ExprKind::Branch(cond, t, f) => {
                let evaluated_cond = evaluate(Rc::new(cond.as_ref().clone()), context)?;

                // we do some basic type checks here,
                // because this function is sometimes called before the type checking
                if evaluated_cond.is_true() {
                    return evaluate(Rc::new(t.as_ref().clone()), context);
                } else if evaluated_cond.is_false() {
                    return evaluate(Rc::new(f.as_ref().clone()), context);
                } else {
                    context.add_error(TypeError::branch_no_boolean(cond.span, todo!()));
                    return Err(());
                }
            },
        }
    }
}
