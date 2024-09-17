use crate::session::MirSession;
use sodigy_high_ir as hir;

pub fn lower_expr(
    expr: &hir::Expr,
    ty: Option<&hir::Type>,
    session: &mut MirSession,
) -> Result<(), ()> {
    todo!()
}
