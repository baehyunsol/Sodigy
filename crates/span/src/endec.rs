use crate::{RenderableSpan, Span, SpanDeriveKind};
use sodigy_endec::{DecodeError, Endec};
use sodigy_file::File;
use sodigy_string::InternedString;

impl Endec for Span {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Span::Lib => {
                buffer.push(0);
            },
            Span::Std => {
                buffer.push(1);
            },
            Span::File(file) => {
                buffer.push(2);
                file.encode_impl(buffer);
            },
            Span::Range { file, start, end } => {
                buffer.push(3);
                file.encode_impl(buffer);
                start.encode_impl(buffer);
                end.encode_impl(buffer);
            },
            Span::Derived { kind, file, start, end } => {
                buffer.push(4);
                kind.encode_impl(buffer);
                file.encode_impl(buffer);
                start.encode_impl(buffer);
                end.encode_impl(buffer);
            },
            Span::Eof(file) => {
                buffer.push(5);
                file.encode_impl(buffer);
            },
            Span::Prelude(s) => {
                buffer.push(6);
                s.encode_impl(buffer);
            },
            Span::Poly(s) => {
                buffer.push(7);
                s.encode_impl(buffer);
            },
            Span::None => {
                buffer.push(8);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((Span::Lib, cursor + 1)),
            Some(1) => Ok((Span::Std, cursor + 1)),
            Some(2) => {
                let (file, cursor) = File::decode_impl(buffer, cursor + 1)?;
                Ok((Span::File(file), cursor))
            },
            Some(3) => {
                let (file, cursor) = File::decode_impl(buffer, cursor + 1)?;
                let (start, cursor) = usize::decode_impl(buffer, cursor)?;
                let (end, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((Span::Range { file, start, end }, cursor))
            },
            Some(4) => {
                let (kind, cursor) = SpanDeriveKind::decode_impl(buffer, cursor + 1)?;
                let (file, cursor) = File::decode_impl(buffer, cursor)?;
                let (start, cursor) = usize::decode_impl(buffer, cursor)?;
                let (end, cursor) = usize::decode_impl(buffer, cursor)?;
                Ok((Span::Derived { kind, file, start, end }, cursor))
            },
            Some(5) => {
                let (file, cursor) = File::decode_impl(buffer, cursor + 1)?;
                Ok((Span::Eof(file), cursor))
            },
            Some(6) => {
                let (p, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((Span::Prelude(p), cursor))
            },
            Some(7) => {
                let (p, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                Ok((Span::Prelude(p), cursor))
            },
            Some(8) => Ok((Span::None, cursor + 1)),
            Some(n @ 9..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
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

        Ok((
            RenderableSpan {
                span,
                auxiliary,
                note,
            },
            cursor,
        ))
    }
}

impl Endec for SpanDeriveKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            SpanDeriveKind::Trivial => {
                buffer.push(0);
            },
            SpanDeriveKind::Pipeline => {
                buffer.push(1);
            },
            SpanDeriveKind::ConstEval => {
                buffer.push(2);
            },
            SpanDeriveKind::DollarIdent => {
                buffer.push(3);
            },
            SpanDeriveKind::Lambda => {
                buffer.push(4);
            },
            SpanDeriveKind::IfLet => {
                buffer.push(5);
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
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((SpanDeriveKind::Trivial, cursor + 1)),
            Some(1) => Ok((SpanDeriveKind::Pipeline, cursor + 1)),
            Some(2) => Ok((SpanDeriveKind::ConstEval, cursor + 1)),
            Some(3) => Ok((SpanDeriveKind::DollarIdent, cursor + 1)),
            Some(4) => Ok((SpanDeriveKind::Lambda, cursor + 1)),
            Some(5) => Ok((SpanDeriveKind::IfLet, cursor + 1)),
            Some(6) => Ok((SpanDeriveKind::FuncDefaultValue, cursor + 1)),
            Some(7) => {
                let (id, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                Ok((SpanDeriveKind::MatchScrutinee(id), cursor))
            },
            Some(8) => Ok((SpanDeriveKind::ConcatPatternRest, cursor + 1)),
            Some(9) => Ok((SpanDeriveKind::ConcatPatternList, cursor + 1)),
            Some(10) => Ok((SpanDeriveKind::FStringToString, cursor + 1)),
            Some(11) => Ok((SpanDeriveKind::FStringConcat, cursor + 1)),
            Some(n @ 12..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
