use crate::{Expr, Let, LetOrigin, Type};
use sodigy_endec::{DecodeError, Endec};
use sodigy_name_analysis::NameOrigin;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

impl Endec for Let {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.keyword_span.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.r#type.encode_impl(buffer);
        self.value.encode_impl(buffer);
        self.origin.encode_impl(buffer);
        self.foreign_names.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (keyword_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (r#type, cursor) = Option::<Type>::decode_impl(buffer, cursor)?;
        let (value, cursor) = Expr::decode_impl(buffer, cursor)?;
        let (origin, cursor) = LetOrigin::decode_impl(buffer, cursor)?;
        let (foreign_names, cursor) = HashMap::<InternedString, (NameOrigin, Span)>::decode_impl(buffer, cursor)?;

        Ok((
            Let {
                keyword_span,
                name,
                name_span,
                r#type,
                value,
                origin,
                foreign_names,
            },
            cursor,
        ))
    }
}

impl Endec for LetOrigin {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            LetOrigin::TopLevel => {
                buffer.push(0);
            },
            LetOrigin::Inline => {
                buffer.push(1);
            },
            LetOrigin::FuncDefaultValue => {
                buffer.push(2);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((LetOrigin::TopLevel, cursor + 1)),
            Some(1) => Ok((LetOrigin::Inline, cursor + 1)),
            Some(2) => Ok((LetOrigin::FuncDefaultValue, cursor + 1)),
            Some(n) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
