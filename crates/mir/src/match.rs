use crate::Session;
use sodigy_hir as hir;

#[derive(Clone, Debug)]
pub struct Match {}

impl Match {
    pub fn from_hir(hir_match: &hir::Match, session: &mut Session) -> Result<Match, ()> {
        todo!()
    }
}
