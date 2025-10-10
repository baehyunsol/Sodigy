use crate::Session;
use sodigy_hir as hir;

#[derive(Clone, Debug)]
pub enum Type {}

impl Type {
    pub fn from_hir(hir_type: &hir::Type, session: &mut Session) -> Result<Type, ()> {
        todo!()
    }
}
