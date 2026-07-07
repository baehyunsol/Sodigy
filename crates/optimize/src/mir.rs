use crate::OptimizeLevel;
use sodigy_mir::Session;

pub fn optimize_mir<'hir, 'mir>(session: Session<'hir, 'mir>, level: OptimizeLevel) -> Session<'hir, 'mir> {
    // TODO: optimize
    session
}
