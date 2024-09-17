use crate::session::MirSession;
use sodigy_high_ir as hir;

pub fn lower_func(
    func: &hir::Func,
    session: &mut MirSession,
) -> Result<(), ()> {
    // 1. walk through all the exprs in the func, and collect local values
    // 2. lower the exprs
    todo!()
}
