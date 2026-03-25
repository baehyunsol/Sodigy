use crate::{
    MonomorphizationInfo,
    PolySpanKind,
    RenderableSpan,
    Span,
    SpanId,
    SpanDeriveKind,
};
use sodigy_endec::{DecodeError, Endec};
use sodigy_string::InternedString;

impl Endec for Span {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Span::Range(r) => {
                buffer.push(0);
                r.encode_impl(buffer);
            },
            Span::Monomorphize { id, span } => {
                buffer.push(1);
                id.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Span::Derived { kind, span } => {
                buffer.push(2);
                kind.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Span::Prelude(id) => {
                buffer.push(3);
                id.encode_impl(buffer);
            },
            Span::Poly { name, kind } => {
                buffer.push(4);
                name.encode_impl(buffer);
                kind.encode_impl(buffer);
            },
            Span::Std => {
                buffer.push(5);
            },
            Span::Lib => {
                buffer.push(6);
            },
            Span::None => {
                buffer.push(7);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (r, cursor) = SpanId::decode_impl(buffer, cursor + 1)?;
                Ok((Span::Range(r), cursor))
            },
            Some(1) => {
                let (id, cursor) = u64::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Box::<Span>::decode_impl(buffer, cursor)?;
                Ok((Span::Monomorphize { id, span }, cursor))
            },
            Some(2) => {
                let (kind, cursor) = SpanDeriveKind::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Box::<Span>::decode_impl(buffer, cursor)?;
                Ok((Span::Derived { kind, span }, cursor))
            },
            Some(3) => {
                let (id, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((Span::Prelude(id), cursor))
            },
            Some(4) => {
                let (name, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                let (kind, cursor) = PolySpanKind::decode_impl(buffer, cursor)?;
                Ok((Span::Poly { name, kind }, cursor))
            },
            Some(5) => Ok((Span::Std, cursor + 1)),
            Some(6) => Ok((Span::Lib, cursor + 1)),
            Some(7) => Ok((Span::None, cursor + 1)),
            Some(n @ 8..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for SpanId {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.0.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (id, cursor) = u128::decode_impl(buffer, cursor)?;
        Ok((SpanId(id), cursor))
    }
}

impl Endec for RenderableSpan {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.span.encode_impl(buffer);
        self.auxiliary.encode_impl(buffer);
        self.note.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (auxiliary, cursor) = bool::decode_impl(buffer, cursor)?;
        let (note, cursor) = Option::<String>::decode_impl(buffer, cursor)?;

        Ok((RenderableSpan { span, auxiliary, note }, cursor))
    }
}

impl Endec for SpanDeriveKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            SpanDeriveKind::Pipeline => {
                buffer.push(0);
            },
            SpanDeriveKind::ConstEval => {
                buffer.push(1);
            },
            SpanDeriveKind::ExprInPattern => {
                buffer.push(2);
            },
            SpanDeriveKind::Lambda => {
                buffer.push(3);
            },
            SpanDeriveKind::IfLet => {
                buffer.push(4);
            },
            SpanDeriveKind::LetPattern(id) => {
                buffer.push(5);
                id.encode_impl(buffer);
            },
            SpanDeriveKind::FuncDefaultValue => {
                buffer.push(6);
            },
            SpanDeriveKind::MatchScrutinee(id) => {
                buffer.push(7);
                id.encode_impl(buffer);
            },
            SpanDeriveKind::ConcatPatternRest => {
                buffer.push(8);
            },
            SpanDeriveKind::ConcatPatternList => {
                buffer.push(9);
            },
            SpanDeriveKind::FStringToString => {
                buffer.push(10);
            },
            SpanDeriveKind::FStringConcat => {
                buffer.push(11);
            },
            SpanDeriveKind::ConvertError => {
                buffer.push(12);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((SpanDeriveKind::Pipeline, cursor + 1)),
            Some(1) => Ok((SpanDeriveKind::ConstEval, cursor + 1)),
            Some(2) => Ok((SpanDeriveKind::ExprInPattern, cursor + 1)),
            Some(3) => Ok((SpanDeriveKind::Lambda, cursor + 1)),
            Some(4) => Ok((SpanDeriveKind::IfLet, cursor + 1)),
            Some(5) => {
                let (id, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                Ok((SpanDeriveKind::LetPattern(id), cursor))
            },
            Some(6) => Ok((SpanDeriveKind::FuncDefaultValue, cursor + 1)),
            Some(7) => {
                let (id, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                Ok((SpanDeriveKind::MatchScrutinee(id), cursor))
            },
            Some(8) => Ok((SpanDeriveKind::ConcatPatternRest, cursor + 1)),
            Some(9) => Ok((SpanDeriveKind::ConcatPatternList, cursor + 1)),
            Some(10) => Ok((SpanDeriveKind::FStringToString, cursor + 1)),
            Some(11) => Ok((SpanDeriveKind::FStringConcat, cursor + 1)),
            Some(12) => Ok((SpanDeriveKind::ConvertError, cursor + 1)),
            Some(n @ 13..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for PolySpanKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            PolySpanKind::Name => {
                buffer.push(0);
            },
            PolySpanKind::Param(i) => {
                buffer.push(1);
                i.encode_impl(buffer);
            },
            PolySpanKind::Return => {
                buffer.push(2);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((PolySpanKind::Name, cursor + 1)),
            Some(1) => {
                let (i, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                Ok((PolySpanKind::Param(i), cursor))
            },
            Some(2) => Ok((PolySpanKind::Return, cursor + 1)),
            Some(n @ 3..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for MonomorphizationInfo {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.id.encode_impl(buffer);
        self.parent.encode_impl(buffer);
        self.info.encode_impl(buffer);
        self.span.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (id, cursor) = u64::decode_impl(buffer, cursor)?;
        let (parent, cursor) = Option::<u64>::decode_impl(buffer, cursor)?;
        let (info, cursor) = String::decode_impl(buffer, cursor)?;
        let (span, cursor) = Span::decode_impl(buffer, cursor)?;

        Ok((MonomorphizationInfo { id, parent, info, span }, cursor))
    }
}
