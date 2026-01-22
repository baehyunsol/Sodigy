use crate::{Expr, Struct, StructInitField, StructField, StructShape, Visibility};
use sodigy_endec::{DecodeError, Endec};
use sodigy_parse::Generic;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

impl Endec for Struct {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.visibility.encode_impl(buffer);
        self.keyword_span.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.generics.encode_impl(buffer);
        self.fields.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (visibility, cursor) = Visibility::decode_impl(buffer, cursor)?;
        let (keyword_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (generics, cursor) = Vec::<Generic>::decode_impl(buffer, cursor)?;
        let (fields, cursor) = Vec::<StructField>::decode_impl(buffer, cursor)?;

        Ok((
            Struct {
                visibility,
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

impl Endec for StructShape {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.name.encode_impl(buffer);
        self.fields.encode_impl(buffer);
        self.generics.encode_impl(buffer);
        self.associated_funcs.encode_impl(buffer);
        self.associated_lets.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (fields, cursor) = Vec::<StructField>::decode_impl(buffer, cursor)?;
        let (generics, cursor) = Vec::<Generic>::decode_impl(buffer, cursor)?;
        let (associated_funcs, cursor) = HashMap::<InternedString, (usize, bool, Vec<Span>)>::decode_impl(buffer, cursor)?;
        let (associated_lets, cursor) = HashMap::<InternedString, Span>::decode_impl(buffer, cursor)?;
        Ok((StructShape {
            name,
            fields,
            generics,
            associated_funcs,
            associated_lets,
        }, cursor))
    }
}
