use super::AST;
use crate::session::LocalParseSession;

mod clean_up_blocks;
mod resolve_recursive_funcs_in_block;

impl AST {

    pub(crate) fn opt(&mut self, session: &mut LocalParseSession) {
        // TODO
    }

}
