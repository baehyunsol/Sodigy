/*
use sdg_ast::{Expr, ExprKind, ValueKind};
use std::rc::Rc;

pub fn evaluate(val: &Expr) -> Result<Rc<Expr>, ()> {
    match val.kind {
        ExprKind::Value(v) => match v {
            // TODO: `.clone` is expensive
            ValueKind::String(_)
            | ValueKind::Integer(_)
            | ValueKind::Real(_)
            | ValueKind::Char(_) => Ok(Rc::new(val.clone())),
        },
        // TODO: tail call
        ExprKind::Call(f, args) => {
            let args_eval = Vec::with_capacity(args.len());

            for arg in args.iter() {
                args_eval.push(evaluate(arg)?);
            }

            todo!()
        }
    }
}
*/