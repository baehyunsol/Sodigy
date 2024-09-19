use super::Func;
use crate::expr::lower_expr;
use crate::session::MirSession;
use crate::ty::{Type, lower_ty};
use sodigy_high_ir as hir;

pub fn lower_func(
    func: &hir::Func,
    session: &mut MirSession,
) -> Result<Func, ()> {
    session.start_lowering_func(func.uid);
    let local_values = session.register_local_values(func)?;
    let return_value = lower_expr(
        &func.return_value,
        (&func.return_type).as_ref(),
        true,
        session,
    )?;
    let return_type = func.return_type.as_ref().map(|ty| lower_ty());
    let return_type = if let Some(ty) = return_type { ty? } else { Type::HasToBeInferred };

    let mut result = Func {
        name: func.name,
        return_value,
        return_type,
        local_values,
        uid: func.uid,
    };
    result.init_local_value_dependency_graphs(
        session,
    )?;

    session.end_lowering_func();
    Ok(result)
}
