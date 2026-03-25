use crate::{File, ModulePath};
use sodigy_endec::{DecodeError, Endec};

impl Endec for File {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.0.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (id, cursor) = u32::decode_impl(buffer, cursor)?;
        Ok((File(id), cursor))
    }
}

impl Endec for ModulePath {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.path.encode_impl(buffer);
        self.is_std.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (path, cursor) = Vec::<String>::decode_impl(buffer, cursor)?;
        let (is_std, cursor) = bool::decode_impl(buffer, cursor)?;

        Ok((ModulePath { path, is_std }, cursor))
    }
}
