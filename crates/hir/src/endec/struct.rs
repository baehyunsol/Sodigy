use crate::{Expr, Struct, StructInitField, StructField, Public, Type};
use sodigy_endec::{DecodeError, Endec};
use sodigy_parse::GenericDef;
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Struct {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.public.encode_impl(buffer);
        self.keyword_span.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.generics.encode_impl(buffer);
        self.fields.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (public, cursor) = Public::decode_impl(buffer, cursor)?;
        let (keyword_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (generics, cursor) = Vec::<GenericDef>::decode_impl(buffer, cursor)?;
        let (fields, cursor) = Vec::<StructField<Type>>::decode_impl(buffer, cursor)?;

        Ok((
            Struct {
                public,
                keyword_span,
                name,
                name_span,
                generics,
                fields,
            },
            cursor,
        ))
    }
}

impl Endec for StructInitField {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.value.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (value, cursor) = Expr::decode_impl(buffer, cursor)?;

        Ok((
            StructInitField {
                name,
                name_span,
                value,
            },
            cursor,
        ))
    }
}
