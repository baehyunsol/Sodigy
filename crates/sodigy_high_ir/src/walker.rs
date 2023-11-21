use crate::expr::{self as hir, Expr, ExprKind};
use crate::func::{Arg, Func, FuncDeco};

pub enum WalkState {
    QuitImmediate,
    Quit,
    Continue,
}

pub trait MutWalkerState {
    fn get_walk_state(&self) -> WalkState {
        WalkState::Continue
    }
}

pub struct EmptyMutWalkerState {}

impl MutWalkerState for EmptyMutWalkerState {}

pub fn mut_walker_func<Ctxt: MutWalkerState, F: Fn(&mut Expr, &mut Ctxt)>(f: &mut Func, c: &mut Ctxt, worker: &Box<F>) {
    mut_walker_expr(&mut f.ret_val, c, worker);

    if let WalkState::Quit = c.get_walk_state() {
        return;
    }

    if let Some(args) = &mut f.args {
        for Arg { ty, .. } in args.iter_mut() {
            if let Some(ty) = ty {
                mut_walker_expr(&mut ty.0, c, worker);
            }
        }

        if let WalkState::Quit = c.get_walk_state() {
            return;
        }
    }

    if let Some(ty) = &mut f.ret_ty {
        mut_walker_expr(&mut ty.0, c, worker);
    }

    if let WalkState::Quit = c.get_walk_state() {
        return;
    }

    f.decorators.mut_walker(c, worker);
}

pub fn mut_walker_expr<Ctxt: MutWalkerState, F: Fn(&mut Expr, &mut Ctxt)>(e: &mut Expr, c: &mut Ctxt, worker: &Box<F>) {
    worker(e, c);

    if let WalkState::QuitImmediate = c.get_walk_state() {
        return;
    }

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
        ExprKind::Scope(hir::Scope {
            defs,
            value,

            // it's just for type-checking.
            // we don't do anything on this
            original_patterns: _,
            ..
        }) => {
            mut_walker_expr(value, c, worker);

            for hir::LocalDef { value, .. } in defs.iter_mut() {
                mut_walker_expr(value, c, worker);
            }
        },
        ExprKind::Match(hir::Match { arms, value }) => {
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
            for hir::BranchArm { cond, let_bind, value } in arms.iter_mut() {
                mut_walker_expr(value, c, worker);

                if let Some(cond) = cond {
                    mut_walker_expr(cond, c, worker);
                }

                if let Some(let_bind) = let_bind {
                    mut_walker_expr(let_bind, c, worker);
                }
            }
        },
        ExprKind::StructInit(hir::StructInit { struct_, fields }) => {
            mut_walker_expr(struct_, c, worker);

            for hir::StructInitField { value, .. } in fields.iter_mut() {
                mut_walker_expr(value, c, worker);
            }
        },
        ExprKind::Path { head, .. } => {
            mut_walker_expr(head, c, worker);
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

impl FuncDeco {
    pub fn mut_walker<Ctxt: MutWalkerState, F: Fn(&mut Expr, &mut Ctxt)>(&mut self, c: &mut Ctxt, worker: &Box<F>) {
        // TODO
    }
}
