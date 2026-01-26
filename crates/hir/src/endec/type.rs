use crate::{Path, Type, TypeAssertion};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;

impl Endec for Type {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Type::Path(p) => {
                buffer.push(0);
                p.encode_impl(buffer);
            },
            Type::Param { constructor, args, group_span } => {
                buffer.push(1);
                constructor.encode_impl(buffer);
                args.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            Type::Tuple { types, group_span } => {
                buffer.push(2);
                types.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            Type::Func { fn_constructor, group_span, params, r#return } => {
                buffer.push(3);
                fn_constructor.encode_impl(buffer);
                group_span.encode_impl(buffer);
                params.encode_impl(buffer);
                r#return.encode_impl(buffer);
            },
            Type::Wildcard(span) => {
                buffer.push(4);
                span.encode_impl(buffer);
            },
            Type::Never(span) => {
                buffer.push(5);
                span.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (path, cursor) = Path::decode_impl(buffer, cursor + 1)?;
                Ok((Type::Path(path), cursor))
            },
            Some(1) => {
                let (constructor, cursor) = Path::decode_impl(buffer, cursor + 1)?;
                let (args, cursor) = Vec::<Type>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Type::Param { constructor, args, group_span }, cursor))
            },
            Some(2) => {
                let (types, cursor) = Vec::<Type>::decode_impl(buffer, cursor + 1)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Type::Tuple { types, group_span }, cursor))
            },
            Some(3) => {
                let (fn_constructor, cursor) = Path::decode_impl(buffer, cursor + 1)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (params, cursor) = Vec::<Type>::decode_impl(buffer, cursor)?;
                let (r#return, cursor) = Box::<Type>::decode_impl(buffer, cursor)?;
                Ok((Type::Func { fn_constructor, group_span, params, r#return }, cursor))
            },
            Some(4) => {
                let (span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Type::Wildcard(span), cursor))
            },
            Some(5) => {
                let (span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Type::Never(span), cursor))
            },
            Some(n @ 6..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for TypeAssertion {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.name_span.encode_impl(buffer);
        self.type_span.encode_impl(buffer);
        self.r#type.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (type_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (r#type, cursor) = Type::decode_impl(buffer, cursor)?;

        Ok((
            TypeAssertion {
                name_span,
                type_span,
                r#type,
            },
            cursor,
        ))
    }
}
