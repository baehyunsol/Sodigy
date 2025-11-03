use crate::{CallArg, Expr, Func, FuncArgDef, FuncOrigin, Public, Type};
use sodigy_endec::{DecodeError, Endec};
use sodigy_name_analysis::{IdentWithOrigin, NameOrigin, UseCount};
use sodigy_parse::GenericDef;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

impl Endec for Func {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.public.encode_impl(buffer);
        self.keyword_span.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.generics.encode_impl(buffer);
        self.args.encode_impl(buffer);
        self.r#type.encode_impl(buffer);
        self.value.encode_impl(buffer);
        self.origin.encode_impl(buffer);
        self.foreign_names.encode_impl(buffer);
        self.use_counts.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (public, cursor) = Public::decode_impl(buffer, cursor)?;
        let (keyword_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (generics, cursor) = Vec::<GenericDef>::decode_impl(buffer, cursor)?;
        let (args, cursor) = Vec::<FuncArgDef<Type>>::decode_impl(buffer, cursor)?;
        let (r#type, cursor) = Option::<Type>::decode_impl(buffer, cursor)?;
        let (value, cursor) = Expr::decode_impl(buffer, cursor)?;
        let (origin, cursor) = FuncOrigin::decode_impl(buffer, cursor)?;
        let (foreign_names, cursor) = HashMap::<InternedString, (NameOrigin, Span)>::decode_impl(buffer, cursor)?;
        let (use_counts, cursor) = HashMap::<InternedString, UseCount>::decode_impl(buffer, cursor)?;

        Ok((
            Func {
                public,
                keyword_span,
                name,
                name_span,
                generics,
                args,
                r#type,
                value,
                origin,
                foreign_names,
                use_counts,
            },
            cursor,
        ))
    }
}

impl<T: Endec> Endec for FuncArgDef<T> {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.r#type.encode_impl(buffer);
        self.default_value.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (r#type, cursor) = Option::<T>::decode_impl(buffer, cursor)?;
        let (default_value, cursor) = Option::<IdentWithOrigin>::decode_impl(buffer, cursor)?;

        Ok((
            FuncArgDef {
                name,
                name_span,
                r#type,
                default_value,
            },
            cursor,
        ))
    }
}

impl Endec for FuncOrigin {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            FuncOrigin::TopLevel => {
                buffer.push(0);
            },
            FuncOrigin::Inline => {
                buffer.push(1);
            },
            FuncOrigin::Lambda => {
                buffer.push(2);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((FuncOrigin::TopLevel, cursor + 1)),
            Some(1) => Ok((FuncOrigin::Inline, cursor + 1)),
            Some(2) => Ok((FuncOrigin::Lambda, cursor + 1)),
            Some(n) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for CallArg {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.keyword.encode_impl(buffer);
        self.arg.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (keyword, cursor) = Option::<(InternedString, Span)>::decode_impl(buffer, cursor)?;
        let (arg, cursor) = Expr::decode_impl(buffer, cursor)?;
        Ok((CallArg { keyword, arg }, cursor))
    }
}
