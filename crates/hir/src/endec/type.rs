use crate::Type;
use sodigy_endec::{DecodeError, Endec};
use sodigy_name_analysis::IdentWithOrigin;
use sodigy_parse::Field;
use sodigy_span::Span;

impl Endec for Type {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Type::Identifier(id) => {
                buffer.push(0);
                id.encode_impl(buffer);
            },
            Type::Path { id, fields } => {
                buffer.push(1);
                id.encode_impl(buffer);
                fields.encode_impl(buffer);
            },
            Type::Param { r#type, args, group_span } => {
                buffer.push(2);
                r#type.encode_impl(buffer);
                args.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            Type::Tuple { types, group_span } => {
                buffer.push(3);
                types.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            Type::Func { fn_span, group_span, args, r#return } => {
                buffer.push(4);
                fn_span.encode_impl(buffer);
                group_span.encode_impl(buffer);
                args.encode_impl(buffer);
                r#return.encode_impl(buffer);
            },
            Type::Wildcard(span) => {
                buffer.push(5);
                span.encode_impl(buffer);
            },
            Type::Never(span) => {
                buffer.push(6);
                span.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (id, cursor) = IdentWithOrigin::decode_impl(buffer, cursor + 1)?;
                Ok((Type::Identifier(id), cursor))
            },
            Some(1) => {
                let (id, cursor) = IdentWithOrigin::decode_impl(buffer, cursor + 1)?;
                let (fields, cursor) = Vec::<Field>::decode_impl(buffer, cursor)?;
                Ok((Type::Path { id, fields }, cursor))
            },
            Some(2) => {
                let (r#type, cursor) = Box::<Type>::decode_impl(buffer, cursor + 1)?;
                let (args, cursor) = Vec::<Type>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Type::Param { r#type, args, group_span }, cursor))
            },
            Some(3) => {
                let (types, cursor) = Vec::<Type>::decode_impl(buffer, cursor + 1)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Type::Tuple { types, group_span }, cursor))
            },
            Some(4) => {
                let (fn_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (args, cursor) = Vec::<Type>::decode_impl(buffer, cursor)?;
                let (r#return, cursor) = Box::<Type>::decode_impl(buffer, cursor)?;
                Ok((Type::Func { fn_span, group_span, args, r#return }, cursor))
            },
            Some(5) => {
                let (span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Type::Wildcard(span), cursor))
            },
            Some(6) => {
                let (span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Type::Never(span), cursor))
            },
            Some(n @ 7..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
