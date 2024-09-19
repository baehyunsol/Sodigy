#![deny(unused_imports)]

use sodigy_high_ir::HirSession;

mod error;
mod expr;
mod func;
mod session;
mod ty;
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
    }
}
