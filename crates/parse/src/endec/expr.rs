use crate::{Expr, Field};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Expr {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        todo!()
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        todo!()
    }
}

impl Endec for Field {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Field::Name { name, name_span, dot_span, is_from_alias } => {
                buffer.push(0);
                name.encode_impl(buffer);
                name_span.encode_impl(buffer);
                dot_span.encode_impl(buffer);
                is_from_alias.encode_impl(buffer);
            },
            Field::Index(n) => {
                buffer.push(1);
                n.encode_impl(buffer);
            },
            Field::Range(a, b) => {
                buffer.push(2);
                a.encode_impl(buffer);
                b.encode_impl(buffer);
            },
            Field::Variant => {
                buffer.push(3);
            },
            Field::Constructor => {
                buffer.push(4);
            },
            Field::Payload => {
                buffer.push(5);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (name, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (dot_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (is_from_alias, cursor) = bool::decode_impl(buffer, cursor)?;
                Ok((
                    Field::Name {
                        name,
                        name_span,
                        dot_span,
                        is_from_alias,
                    },
                    cursor,
                ))
            },
            Some(1) => {
                let (n, cursor) = i64::decode_impl(buffer, cursor + 1)?;
                Ok((Field::Index(n), cursor))
            },
            Some(2) => {
                let (a, cursor) = i64::decode_impl(buffer, cursor + 1)?;
                let (b, cursor) = i64::decode_impl(buffer, cursor)?;
                Ok((Field::Range(a, b), cursor))
            },
            Some(3) => Ok((Field::Variant, cursor + 1)),
            Some(4) => Ok((Field::Constructor, cursor + 1)),
            Some(5) => Ok((Field::Payload, cursor + 1)),
            Some(n @ 6..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
