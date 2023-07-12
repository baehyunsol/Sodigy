use super::{AST, ASTError};

mod clean_up_blocks;

impl AST {

    pub fn opt(&mut self) -> Result<(), ASTError> {
        // TODO

        Ok(())
    }

    // 1. If there's a lambda function, it creates a new function named `__LAMBDA_XXXX`.
    //   - All the uses of the lambda are replaced by `__LAMBDA_XXXX`.
    // 2. If there's a closure... how do I implement closures?
    pub fn make_lambdas_and_closures_static(&mut self) {}
}
