use crate::expr::LocalDef;
use crate::session::HirSession;
use sodigy_ast as ast;

pub struct Pattern {}

pub fn lower_ast_local_def(
    local_def: &ast::LocalDef,
    session: &mut HirSession,
) -> Result<LocalDef, ()> {
    todo!()
}
