use super::{AST, ASTError};

/*
 * Name Precedence
 *
 * 1. Name Scope (defs in block_expr, args in func, args in lambda)
 *   - Close -> Far
 * 2. `use`s and `def`s
 *   - Same names not allowed
 * 3. preludes
 *
 * When it sees `use A.B.C;`, it doesn't care whether `A` is valid or not.
 * It just assumes that everything is fine. Another checker will alert the programmer
 * if `A` is invalid. Then it halts anyway...
 */

impl AST {

    pub fn resolve_names(&mut self) -> Result<(), ASTError> {
        todo!()
    }

}