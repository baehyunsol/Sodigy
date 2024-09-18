use super::Func;
use crate::session::MirSession;
use sodigy_high_ir as hir;

pub fn lower_func(
    func: &hir::Func,
    session: &mut MirSession,
) -> Result<Func, ()> {
    session.start_lowering_func(func.uid);
    session.register_local_values(func);

    session.end_lowering_func();
    Ok(Func {
        name: func.name,
    })
}
