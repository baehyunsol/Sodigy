use super::Def;
use crate::expr::lower::lower_hir_expr_without_types;
use crate::prelude::PreludeData;
use crate::session::MirSession;
use crate::ty::{Type, try_convert_expr_to_ty};
use sodigy_high_ir as hir;
use sodigy_intern::InternedString;
use std::collections::HashMap;

pub fn lower_hir_func(
    func: &hir::Func,
    session: &mut MirSession,
    preludes: &HashMap<InternedString, PreludeData>,
) -> Result<Def, ()> {
    match &func.kind {
        hir::FuncKind::Normal => {
            let return_ty = match &func.return_ty {
                Some(ty) => {
                    let lowered_ty = lower_hir_expr_without_types(
                        ty.as_expr(),
                        session,
                        preludes,
                    );

                    match try_convert_expr_to_ty(&lowered_ty) {
                        Some(ty) => ty,
                        None => Type::HasToBeConverted(Box::new(lowered_ty)),
                    }
                },
                None => Type::HasToBeInfered,
            };

            let return_val = lower_hir_expr_without_types(
                &func.return_val,
                session,
                preludes,
            );

            todo!()
        },
        _ => todo!(),
    }
}
