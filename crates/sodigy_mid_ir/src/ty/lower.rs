use super::Type;
use crate::error::MirError;
use crate::expr::{ExprKind, lower_expr};
use crate::session::MirSession;
use sodigy_high_ir as hir;
use sodigy_prelude as prelude;
use sodigy_session::SodigySession;

pub fn lower_ty(
    ty: &hir::Type,
    session: &mut MirSession,
) -> Result<Type, ()> {
    let ty_expr = lower_expr(
        ty.as_expr(),
        None,
        false,
        session,
    )?;

    match &ty_expr.kind {
        ExprKind::Integer(_) => {
            let expected_ty_rendered = Type::from_uid(prelude::TYPE.1).render_error(session);
            let got_ty_rendered = Type::from_uid(prelude::TYPE.1).render_error(session);

            session.push_error(MirError::type_error(
                &ty_expr,
                expected_ty_rendered,
                got_ty_rendered,
            ));
            Err(())
        },
        ExprKind::Object(uid) => Ok(Type::Simple(*uid)),

        // TODO: check whether it's generic or not
        //       if so, return Type::Generic,
        //       otherwise, it's a dependent type
        ExprKind::LocalValue { .. } => Ok(Type::HasToBeLowered(Box::new(ty_expr))),

        // TODO: why not just lower here?
        ExprKind::Call { .. } => Ok(Type::HasToBeLowered(Box::new(ty_expr))),
    }
}
