use crate::{Bytecode, Session};
use sodigy_mir as mir;

// It returns `Vec<Bytecode>` instead of inserting it to `session.lets` because
// it's not sure whether `mir_let` is top-level or not.
pub fn lower_mir_let(mir_let: &mir::Let, session: &mut Session) -> Vec<Bytecode> {
    todo!()
}
