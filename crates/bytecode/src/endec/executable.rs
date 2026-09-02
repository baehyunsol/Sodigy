use crate::{Bytecode, Executable, ExprHash, Value};
use sodigy_endec::{DecodeError, Endec};
use std::collections::HashMap;

impl Endec for Executable {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.data.encode_impl(buffer);
        self.code.encode_impl(buffer);
        self.main_entry.encode_impl(buffer);
        self.asserts.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (data, cursor) = HashMap::<ExprHash, Value>::decode_impl(buffer, cursor)?;
        let (code, cursor) = Vec::<Bytecode>::decode_impl(buffer, cursor)?;
        let (main_entry, cursor) = Option::<usize>::decode_impl(buffer, cursor)?;
        let (asserts, cursor) = Vec::<(String, usize)>::decode_impl(buffer, cursor)?;

        Ok((
            Executable {
                data,
                code,
                main_entry,
                asserts,
            },
            cursor,
        ))
    }
}
