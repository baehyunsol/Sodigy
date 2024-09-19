use crate::{Scope, ScopedLet};
use crate::expr::{self as hir, Expr, ExprKind};
use crate::func::{Arg, Func};
use sodigy_attribute::{Attribute, Decorator};

pub fn walker_func<Ctxt, F: Fn(&Expr, &mut Ctxt)>(f: &Func, c: &mut Ctxt, worker: &Box<F>) {
    walker_expr(&f.return_value, c, worker);

    if let Some(args) = &f.args {
        for Arg { ty, attributes, .. } in args.iter() {
            if let Some(ty) = ty {
                walker_expr(&ty.0, c, worker);
            }

            for attribute in attributes.iter() {
                if let Attribute::Decorator(d) = attribute {
                    walker_decorator(d, c, worker);
                }
            }
        }
    }

    if let Some(ty) = &f.return_type {
        walker_expr(ty.as_expr(), c, worker);
    }

    for attribute in f.attributes.iter() {
        if let Attribute::Decorator(d) = attribute {
            walker_decorator(d, c, worker);
        }
    }
}

pub fn walker_expr<Ctxt, F: Fn(&Expr, &mut Ctxt)>(e: &Expr, c: &mut Ctxt, worker: &Box<F>) {
    worker(e, c);

    match &e.kind {
        ExprKind::Identifier(_)
        | ExprKind::Integer(_)
        | ExprKind::Ratio(_)
        | ExprKind::Char(_)
        | ExprKind::String { .. } => { /* nop */ },
        ExprKind::Call {
            func, args,
        } => {
            walker_expr(func, c, worker);

            for arg in args.iter() {
                walker_expr(arg, c, worker);
            }
        },
        ExprKind::List(elems)
        | ExprKind::Tuple(elems)
        | ExprKind::Format(elems) => {
            for elem in elems.iter() {
                walker_expr(elem, c, worker);
            }
        },
        ExprKind::Scope(Scope {
            lets,
            value,

            // it's just for type-checking.
            // we don't do anything on this
            original_patterns: _,
            ..
        }) => {
            walker_expr(value, c, worker);

            for ScopedLet { value, .. } in lets.iter() {
                walker_expr(value, c, worker);
            }
        },
        ExprKind::Match(hir::Match { arms, value, .. }) => {
            walker_expr(value, c, worker);

            for hir::MatchArm { value, guard, .. } in arms.iter() {
                walker_expr(value, c, worker);

                if let Some(g) = guard {
                    walker_expr(g, c, worker);
                }
            }
        },
        ExprKind::Lambda(hir::Lambda {
            args, value, captured_values, ..
        }) => {
            walker_expr(value, c, worker);

            for Arg { ty, .. } in args.iter() {
                if let Some(ty) = ty {
                    walker_expr(ty.as_expr(), c, worker);
                }
            }

            for value in captured_values.iter() {
                walker_expr(value, c, worker);
            }
        },
        ExprKind::Branch(hir::Branch { arms }) => {
            for hir::BranchArm { cond, value } in arms.iter() {
                walker_expr(value, c, worker);

                if let Some(cond) = cond {
                    walker_expr(cond, c, worker);
                }
            }
        },
        ExprKind::StructInit(hir::StructInit { struct_, fields }) => {
            walker_expr(struct_, c, worker);

            for hir::StructInitField { value, .. } in fields.iter() {
                walker_expr(value, c, worker);
            }
        },
        ExprKind::Field { pre, .. } => {
            walker_expr(pre, c, worker);
        },
        ExprKind::PrefixOp(_, value)
        | ExprKind::PostfixOp(_, value) => {
            walker_expr(value, c, worker);
        },
        ExprKind::InfixOp(_, lhs, rhs) => {
            walker_expr(lhs, c, worker);
            walker_expr(rhs, c, worker);
        },
    }
}

pub fn walker_decorator<Ctxt, F: Fn(&Expr, &mut Ctxt)>(d: &Decorator<Expr>, c: &mut Ctxt, worker: &Box<F>) {
    if let Some(args) = &d.args {
        for arg in args.iter() {
            walker_expr(arg, c, worker);
        }
    }
}
