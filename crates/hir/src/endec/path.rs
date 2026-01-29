use crate::{Path, Type};
use sodigy_endec::{DecodeError, Endec};
use sodigy_name_analysis::IdentWithOrigin;
use sodigy_parse::Field;

impl Endec for Path {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.id.encode_impl(buffer);
        self.fields.encode_impl(buffer);
        self.types.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (id, cursor) = IdentWithOrigin::decode_impl(buffer, cursor)?;
        let (fields, cursor) = Vec::<Field>::decode_impl(buffer, cursor)?;
        let (types, cursor) = Vec::<Option<Vec<Type>>>::decode_impl(buffer, cursor)?;

        Ok((
            Path {
                id,
                fields,
                types,
            },
            cursor,
        ))
    }
}
