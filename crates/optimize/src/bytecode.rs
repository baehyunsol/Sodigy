use crate::OptimizeLevel;
use sodigy_bytecode::Session;

pub fn optimize_bytecode<'hir, 'mir>(mut session: Session<'hir, 'mir>, level: OptimizeLevel) -> Session<'hir, 'mir> {
    // TODO
    session
}
