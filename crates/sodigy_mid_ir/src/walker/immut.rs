use crate::expr::{Expr, ExprKind, MirFunc};

pub fn walker_expr<Ctxt, F: Fn(&Expr, &mut Ctxt, bool)>(
    e: &Expr,
    c: &mut Ctxt,
    worker: &Box<F>,

    // used to distinguish values in branches
    is_conditional: bool,
) {
    worker(e, c, is_conditional);

    match &e.kind {
        ExprKind::Integer(_)
        | ExprKind::LocalValue { .. }
        | ExprKind::Object(_) => { /* nop */ },
        ExprKind::Call {
            func,
            args,
            ..
        } => {
            if let MirFunc::Dynamic(f) = func {
                walker_expr(f.as_ref(), c, worker, is_conditional);
            }

            for arg in args.iter() {
                walker_expr(arg, c, worker, is_conditional);
            }
        },
    }
}
