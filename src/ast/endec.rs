use super::AST;
use crate::endec::{Endec, EndecError};

impl Endec for AST {
    fn encode(&self, buffer: &mut Vec<u8>) {
        todo!()
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        todo!()
    }
}
