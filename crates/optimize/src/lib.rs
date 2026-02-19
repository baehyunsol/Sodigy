use sodigy_bytecode::Session as BytecodeSession;
use sodigy_mir::Session as MirSession;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptimizeLevel {
    None,
    Mild,
    Extreme,
}

pub fn optimize_mir(mir_session: MirSession, level: OptimizeLevel) -> MirSession {
    // TODO: optimize
    mir_session
}

pub fn optimize_bytecode(bytecode_session: BytecodeSession, level: OptimizeLevel) -> BytecodeSession {
    // TODO: optimize
    bytecode_session
}
