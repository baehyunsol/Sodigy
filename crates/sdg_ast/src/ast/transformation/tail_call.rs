use super::super::AST;
use crate::expr::{Expr, ExprKind, MatchBranch, TailCall};

impl AST {
    pub fn mark_tail_calls(&mut self) {
        for func in self.defs.values_mut() {
            func.ret_val.mark_tail_call();
        }
    }
}

impl Expr {
    pub fn mark_tail_call(&mut self) {
        match &mut self.kind {
            ExprKind::Call(_, _, tail) => {
                *tail = TailCall::Tail;
            },
            ExprKind::Branch(_, t, f) => {
                t.mark_tail_call();
                f.mark_tail_call();
            },
            ExprKind::Match(_, branches, _) => {
                for MatchBranch { value, .. } in branches.iter_mut() {
                    value.mark_tail_call();
                }
            },
            _ => {}
        }
    }
}
