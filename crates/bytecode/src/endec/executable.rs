use crate::{Bytecode, Executable};
use sodigy_endec::{DecodeError, Endec};

impl Endec for Executable {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.asserts.encode_impl(buffer);
        self.bytecodes.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (asserts, cursor) = Vec::<(String, usize)>::decode_impl(buffer, cursor)?;
        let (bytecodes, cursor) = Vec::<Bytecode>::decode_impl(buffer, cursor)?;

        Ok((
            Executable {
                asserts,
                bytecodes,
            },
            cursor,
        ))
    }
}
