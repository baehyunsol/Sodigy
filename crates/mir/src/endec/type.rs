use crate::Type;
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;

impl Endec for Type {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Type::Static(def_span) => {
                buffer.push(0);
                def_span.encode_impl(buffer);
            },
            Type::GenericDef(def_span) => {
                buffer.push(1);
                def_span.encode_impl(buffer);
            },
            Type::Unit(group_span) => {
                buffer.push(2);
                group_span.encode_impl(buffer);
            },
            Type::Never(span) => {
                buffer.push(3);
                span.encode_impl(buffer);
            },
            Type::Param { r#type, args, group_span } => {
                buffer.push(4);
                r#type.encode_impl(buffer);
                args.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            Type::Func { fn_span, group_span, args, r#return } => {
                buffer.push(5);
                fn_span.encode_impl(buffer);
                group_span.encode_impl(buffer);
                args.encode_impl(buffer);
                r#return.encode_impl(buffer);
            },
            Type::Var { def_span, is_return } => {
                buffer.push(6);
                def_span.encode_impl(buffer);
                is_return.encode_impl(buffer);
            },
            Type::GenericInstance { call, generic } => {
                buffer.push(7);
                call.encode_impl(buffer);
                generic.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (def_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Type::Static(def_span), cursor))
            },
            Some(1) => {
                let (def_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Type::GenericDef(def_span), cursor))
            },
            Some(2) => {
                let (group_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Type::Unit(group_span), cursor))
            },
            Some(3) => {
                let (span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Type::Never(span), cursor))
            },
            Some(4) => {
                let (r#type, cursor) = Box::<Type>::decode_impl(buffer, cursor + 1)?;
                let (args, cursor) = Vec::<Type>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Type::Param { r#type, args, group_span }, cursor))
            },
            Some(5) => {
                let (fn_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (args, cursor) = Vec::<Type>::decode_impl(buffer, cursor)?;
                let (r#return, cursor) = Box::<Type>::decode_impl(buffer, cursor)?;
                Ok((Type::Func { fn_span, group_span, args, r#return }, cursor))
            },
            Some(6) => {
                let (def_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                let (is_return, cursor) = bool::decode_impl(buffer, cursor)?;
                Ok((Type::Var { def_span, is_return }, cursor))
            },
            Some(7) => {
                let (call, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                let (generic, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Type::GenericInstance { call, generic }, cursor))
            },
            Some(n @ 8..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
