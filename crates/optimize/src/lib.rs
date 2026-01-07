use sodigy_mir::Session;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptimizeLevel {
    None,
    Mild,
    Extreme,
}

pub fn optimize(mir_session: Session, level: OptimizeLevel) -> Session {
    // TODO: optimize
    mir_session
}
