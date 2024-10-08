use crate::{Scope, ScopedLet};
use crate::expr::{self as hir, Expr, ExprKind};
use crate::func::{Arg, Func};
use sodigy_attribute::{Attribute, Decorator};

pub fn mut_walker_func<Ctxt, F: Fn(&mut Expr, &mut Ctxt)>(f: &mut Func, c: &mut Ctxt, worker: &Box<F>) {
    mut_walker_expr(&mut f.return_value, c, worker);

    if let Some(args) = &mut f.args {
        for Arg { ty, attributes, .. } in args.iter_mut() {
            if let Some(ty) = ty {
                mut_walker_expr(&mut ty.0, c, worker);
            }

            for attribute in attributes.iter_mut() {
                if let Attribute::Decorator(d) = attribute {
                    mut_walker_decorator(d, c, worker);
                }
            }
        }
    }

    if let Some(ty) = &mut f.return_type {
        mut_walker_expr(&mut ty.0, c, worker);
    }

    for attribute in f.attributes.iter_mut() {
        if let Attribute::Decorator(d) = attribute {
            mut_walker_decorator(d, c, worker);
        }
    }
}

pub fn mut_walker_expr<Ctxt, F: Fn(&mut Expr, &mut Ctxt)>(e: &mut Expr, c: &mut Ctxt, worker: &Box<F>) {
    worker(e, c);

    match &mut e.kind {
        ExprKind::Identifier(_)
        | ExprKind::Integer(_)
        | ExprKind::Ratio(_)
        | ExprKind::Char(_)
        | ExprKind::String { .. } => { /* nop */ },
        ExprKind::Call {
            func, args,
        } => {
            mut_walker_expr(func, c, worker);

            for arg in args.iter_mut() {
                mut_walker_expr(arg, c, worker);
            }
        },
        ExprKind::List(elems)
        | ExprKind::Tuple(elems)
        | ExprKind::Format(elems) => {
            for elem in elems.iter_mut() {
                mut_walker_expr(elem, c, worker);
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
            mut_walker_expr(value, c, worker);

            for ScopedLet { value, ty, .. } in lets.iter_mut() {
                mut_walker_expr(value, c, worker);

                if let Some(ty) = ty {
                    mut_walker_expr(&mut ty.0, c, worker);
                }
            }
        },
        ExprKind::Match(hir::Match { arms, value, .. }) => {
            mut_walker_expr(value, c, worker);

            for hir::MatchArm { value, guard, .. } in arms.iter_mut() {
                mut_walker_expr(value, c, worker);

                if let Some(g) = guard {
                    mut_walker_expr(g, c, worker);
                }
            }
        },
        ExprKind::Lambda(hir::Lambda {
            args, value, captured_values, ..
        }) => {
            mut_walker_expr(value, c, worker);

            for Arg { ty, .. } in args.iter_mut() {
                if let Some(ty) = ty {
                    mut_walker_expr(&mut ty.0, c, worker);
                }
            }

            for value in captured_values.iter_mut() {
                mut_walker_expr(value, c, worker);
            }
        },
        ExprKind::Branch(hir::Branch { arms }) => {
            for hir::BranchArm { cond, value } in arms.iter_mut() {
                mut_walker_expr(value, c, worker);

                if let Some(cond) = cond {
                    mut_walker_expr(cond, c, worker);
                }
            }
        },
        ExprKind::StructInit(hir::StructInit { struct_, fields }) => {
            mut_walker_expr(struct_, c, worker);

            for hir::StructInitField { value, .. } in fields.iter_mut() {
                mut_walker_expr(value, c, worker);
            }
        },
        ExprKind::Field { pre, .. } => {
            mut_walker_expr(pre, c, worker);
        },
        ExprKind::PrefixOp(_, value)
        | ExprKind::PostfixOp(_, value) => {
            mut_walker_expr(value, c, worker);
        },
        ExprKind::InfixOp(_, lhs, rhs) => {
            mut_walker_expr(lhs, c, worker);
            mut_walker_expr(rhs, c, worker);
        },
    }
}

pub fn mut_walker_decorator<Ctxt, F: Fn(&mut Expr, &mut Ctxt)>(d: &mut Decorator<Expr>, c: &mut Ctxt, worker: &Box<F>) {
    if let Some(args) = &mut d.args {
        for arg in args.iter_mut() {
            mut_walker_expr(arg, c, worker);
        }
    }
}
