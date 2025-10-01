use crate::Type;
use sodigy_error::Error;

impl Type {
    pub fn check(&self) -> Result<(), Vec<Error>> {
        // It's just an AST of a type signature. There's nothing to check.
        Ok(())
    }
}
