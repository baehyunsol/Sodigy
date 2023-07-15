use super::{AST, ASTError};

mod clean_up_blocks;

impl AST {

    pub fn opt(&mut self) -> Result<(), ASTError> {
        // self.clean_up_blocks();  // TODO

        Ok(())
    }

}
