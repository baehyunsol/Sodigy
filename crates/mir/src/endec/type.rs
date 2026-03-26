use crate::{Dotfish, Type, TypeAssertion};
use sodigy_endec::{DecodeError, Endec};
use sodigy_hir::FuncPurity;
use sodigy_span::Span;

impl Endec for Type {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Type::Data { constructor_def_span, constructor_span, args, group_span } => {
                buffer.push(0);
                constructor_def_span.encode_impl(buffer);
                constructor_span.encode_impl(buffer);
                args.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            Type::Func { fn_span, group_span, params, r#return, purity } => {
                buffer.push(1);
                fn_span.encode_impl(buffer);
                group_span.encode_impl(buffer);
                params.encode_impl(buffer);
                r#return.encode_impl(buffer);
                purity.encode_impl(buffer);
            },
            Type::Never(span) => {
                buffer.push(2);
                span.encode_impl(buffer);
            },
            Type::GenericParam { def_span, span } => {
                buffer.push(3);
                def_span.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Type::Var { def_span, is_return } => {
                buffer.push(4);
                def_span.encode_impl(buffer);
                is_return.encode_impl(buffer);
            },
            Type::GenericArg { call, generic } => {
                buffer.push(5);
                call.encode_impl(buffer);
                generic.encode_impl(buffer);
            },
            Type::Blocked { origin } => {
                buffer.push(6);
                origin.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (constructor_def_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                let (constructor_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (args, cursor) = Option::<Vec<Type>>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;
                Ok((Type::Data { constructor_def_span, constructor_span, args, group_span }, cursor))
            },
            Some(1) => {
                let (fn_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (params, cursor) = Vec::<Type>::decode_impl(buffer, cursor)?;
                let (r#return, cursor) = Box::<Type>::decode_impl(buffer, cursor)?;
                let (purity, cursor) = FuncPurity::decode_impl(buffer, cursor)?;
                Ok((Type::Func { fn_span, group_span, params, r#return, purity }, cursor))
            },
            Some(2) => {
                let (span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Type::Never(span), cursor))
            },
            Some(3) => {
                let (def_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Type::GenericParam { def_span, span }, cursor))
            },
            Some(4) => {
                let (def_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                let (is_return, cursor) = bool::decode_impl(buffer, cursor)?;
                Ok((Type::Var { def_span, is_return }, cursor))
            },
            Some(5) => {
                let (call, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                let (generic, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Type::GenericArg { call, generic }, cursor))
            },
            Some(6) => {
                let (origin, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Type::Blocked { origin }, cursor))
            },
            Some(n @ 7..) => Err(DecodeError::InvalidEnumVariant(*n)),
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

impl Endec for Dotfish {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.types.encode_impl(buffer);
        self.group_span.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (types, cursor) = Vec::<Type>::decode_impl(buffer, cursor)?;
        let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
        Ok((Dotfish { types, group_span }, cursor))
    }
}
