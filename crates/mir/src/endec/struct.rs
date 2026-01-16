use crate::Struct;
use sodigy_endec::{DecodeError, Endec};
use sodigy_hir::Generic;
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Struct {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.generics.encode_impl(buffer);
        self.fields.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (generics, cursor) = Vec::<Generic>::decode_impl(buffer, cursor)?;
        let (fields, cursor) = Vec::<(InternedString, Span)>::decode_impl(buffer, cursor)?;

        Ok((
            Struct {
                name,
                name_span,
                generics,
                fields,
            },
            cursor,
        ))
    }
}
