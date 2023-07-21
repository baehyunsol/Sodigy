use super::{AST, ASTError};
use crate::session::LocalParseSession;

mod clean_up_blocks;

impl AST {

    pub(crate) fn opt(&mut self, session: &mut LocalParseSession) -> Result<(), ASTError> {
        self.clean_up_blocks(session)?;  // TODO: it's not an optimization because it finds cycles in block-defs

        Ok(())
    }

}
