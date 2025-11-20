use crate::{Enum, EnumVariant, EnumVariantFields, StructField, Type, Visibility};
use sodigy_endec::{DecodeError, Endec};
use sodigy_parse::Generic;
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Enum {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.visibility.encode_impl(buffer);
        self.keyword_span.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.generics.encode_impl(buffer);
        self.variants.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (visibility, cursor) = Visibility::decode_impl(buffer, cursor)?;
        let (keyword_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (generics, cursor) = Vec::<Generic>::decode_impl(buffer, cursor)?;
        let (variants, cursor) = Vec::<EnumVariant>::decode_impl(buffer, cursor)?;

        Ok((
            Enum {
                visibility,
                keyword_span,
                name,
                name_span,
                generics,
                variants,
            },
            cursor,
        ))
    }
}

impl Endec for EnumVariant {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.fields.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (fields, cursor) = EnumVariantFields::decode_impl(buffer, cursor)?;

        Ok((
            EnumVariant {
                name,
                name_span,
                fields,
            },
            cursor,
        ))
    }
}

impl Endec for EnumVariantFields {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            EnumVariantFields::None => {
                buffer.push(0);
            },
            EnumVariantFields::Tuple(types) => {
                buffer.push(1);
                types.encode_impl(buffer);
            },
            EnumVariantFields::Struct(fields) => {
                buffer.push(2);
                fields.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((EnumVariantFields::None, cursor + 1)),
            Some(1) => {
                let (types, cursor) = Vec::<Type>::decode_impl(buffer, cursor + 1)?;
                Ok((EnumVariantFields::Tuple(types), cursor))
            },
            Some(2) => {
                let (fields, cursor) = Vec::<StructField>::decode_impl(buffer, cursor + 1)?;
                Ok((EnumVariantFields::Struct(fields), cursor))
            },
            Some(n @ 3..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
