#![deny(unused_imports)]

use sodigy_high_ir::HirSession;

mod error;
mod expr;
mod func;
mod session;
mod ty;
mod walker;
mod warn;

use crate::func::lower_func;
pub use crate::session::MirSession;

pub fn lower_funcs(
    hir_session: &HirSession,
    mir_session: &mut MirSession,
) {
    for func in hir_session.func_defs.values() {
        if let Ok(f) = lower_func(
            func,
            mir_session,
        ) {
            mir_session.func_defs.insert(
                f.uid,
                f,
            );
        }

        else if mir_session.curr_lowering_func.is_some() {
            mir_session.end_lowering_func();
        }
    }
}
