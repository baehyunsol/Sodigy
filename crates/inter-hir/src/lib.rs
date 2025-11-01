mod endec;
mod session;

pub use session::Session;

impl Session {
    pub fn ingest(&mut self, hir_session: sodigy_hir::Session) {
        todo!();
    }
}
