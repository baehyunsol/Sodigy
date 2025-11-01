use crate::Session;
use sodigy_endec::{DecodeError, DumpIr, Endec};
use sodigy_error::{Error, Warning};

impl Endec for Session {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        // changes everytime
        // self.intermediate_dir.encode_impl(buffer);

        self.errors.encode_impl(buffer);
        self.warnings.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
        let (warnings, cursor) = Vec::<Warning>::decode_impl(buffer, cursor)?;

        Ok((
            Session {
                // You have to set this after decoding it.
                intermediate_dir: String::new(),
                errors,
                warnings,
            },
            cursor,
        ))
    }
}

impl DumpIr for Session {
    fn dump_ir(&self) -> Vec<u8> {
        todo!()
    }
}
