use crate::{Struct, StructField};
use sodigy_endec::{DecodeError, Endec};
use sodigy_hir::Generic;
use sodigy_name_analysis::IdentWithOrigin;
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
        let (fields, cursor) = Vec::<StructField>::decode_impl(buffer, cursor)?;

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

impl Endec for StructField {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.default_value.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (default_value, cursor) = Option::<IdentWithOrigin>::decode_impl(buffer, cursor)?;

        Ok((
            StructField {
                name,
                name_span,
                default_value,
            },
            cursor,
        ))
    }
}
