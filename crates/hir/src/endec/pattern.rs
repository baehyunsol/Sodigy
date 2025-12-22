use crate::{Pattern, PatternKind, StructFieldPattern, Type};
use sodigy_endec::{DecodeError, Endec};
use sodigy_number::InternedNumber;
use sodigy_parse::RestPattern;
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Pattern {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.kind.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (name, cursor) = Option::<InternedString>::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;
        let (kind, cursor) = PatternKind::decode_impl(buffer, cursor)?;

        Ok((
            Pattern {
                name,
                name_span,
                kind,
            },
            cursor,
        ))
    }
}

impl Endec for PatternKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            PatternKind::Ident { id, span } => {
                buffer.push(0);
                id.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            PatternKind::Number { n, span } => {
                buffer.push(1);
                n.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            PatternKind::String { binary, s, span } => {
                buffer.push(2);
                binary.encode_impl(buffer);
                s.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            PatternKind::Regex { s, span } => {
                buffer.push(3);
                s.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            PatternKind::Char { ch, span } => {
                buffer.push(4);
                ch.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            PatternKind::Byte { b, span } => {
                buffer.push(5);
                b.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            PatternKind::Path(path) => {
                buffer.push(6);
                path.encode_impl(buffer);
            },
            PatternKind::Struct { r#struct, fields, rest, group_span } => {
                buffer.push(7);
                r#struct.encode_impl(buffer);
                fields.encode_impl(buffer);
                rest.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            PatternKind::TupleStruct { r#struct, elements, rest, group_span } => {
                buffer.push(8);
                r#struct.encode_impl(buffer);
                elements.encode_impl(buffer);
                rest.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            PatternKind::Tuple { elements, rest, group_span } => {
                buffer.push(9);
                elements.encode_impl(buffer);
                rest.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            PatternKind::List { elements, rest, group_span } => {
                buffer.push(10);
                elements.encode_impl(buffer);
                rest.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            PatternKind::Range { lhs, rhs, op_span, is_inclusive } => {
                buffer.push(11);
                lhs.encode_impl(buffer);
                rhs.encode_impl(buffer);
                op_span.encode_impl(buffer);
                is_inclusive.encode_impl(buffer);
            },
            PatternKind::Or { lhs, rhs, op_span } => {
                buffer.push(12);
                lhs.encode_impl(buffer);
                rhs.encode_impl(buffer);
                op_span.encode_impl(buffer);
            },
            PatternKind::Wildcard(span) => {
                buffer.push(13);
                span.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (id, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Ident { id, span }, cursor))
            },
            Some(1) => {
                let (n, cursor) = InternedNumber::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Number { n, span }, cursor))
            },
            Some(2) => {
                let (binary, cursor) = bool::decode_impl(buffer, cursor + 1)?;
                let (s, cursor) = InternedString::decode_impl(buffer, cursor)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::String { binary, s, span }, cursor))
            },
            Some(3) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Regex { s, span }, cursor))
            },
            Some(4) => {
                let (ch, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Char { ch, span }, cursor))
            },
            Some(5) => {
                let (b, cursor) = u8::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Byte { b, span }, cursor))
            },
            Some(6) => {
                let (path, cursor) = Vec::<(InternedString, Span)>::decode_impl(buffer, cursor + 1)?;
                Ok((PatternKind::Path(path), cursor))
            },
            Some(7) => {
                let (r#struct, cursor) = Vec::<(InternedString, Span)>::decode_impl(buffer, cursor + 1)?;
                let (fields, cursor) = Vec::<StructFieldPattern>::decode_impl(buffer, cursor)?;
                let (rest, cursor) = Option::<RestPattern>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Struct { r#struct, fields, rest, group_span }, cursor))
            },
            Some(8) => {
                let (r#struct, cursor) = Vec::<(InternedString, Span)>::decode_impl(buffer, cursor + 1)?;
                let (elements, cursor) = Vec::<Pattern>::decode_impl(buffer, cursor)?;
                let (rest, cursor) = Option::<RestPattern>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::TupleStruct { r#struct, elements, rest, group_span }, cursor))
            },
            Some(9) => {
                let (elements, cursor) = Vec::<Pattern>::decode_impl(buffer, cursor + 1)?;
                let (rest, cursor) = Option::<RestPattern>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Tuple { elements, rest, group_span }, cursor))
            },
            Some(10) => {
                let (elements, cursor) = Vec::<Pattern>::decode_impl(buffer, cursor + 1)?;
                let (rest, cursor) = Option::<RestPattern>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::List { elements, rest, group_span }, cursor))
            },
            Some(11) => {
                let (lhs, cursor) = Option::<Box<Pattern>>::decode_impl(buffer, cursor + 1)?;
                let (rhs, cursor) = Option::<Box<Pattern>>::decode_impl(buffer, cursor)?;
                let (op_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (is_inclusive, cursor) = bool::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Range { lhs, rhs, op_span, is_inclusive }, cursor))
            },
            Some(12) => {
                let (lhs, cursor) = Box::<Pattern>::decode_impl(buffer, cursor + 1)?;
                let (rhs, cursor) = Box::<Pattern>::decode_impl(buffer, cursor)?;
                let (op_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Or { lhs, rhs, op_span }, cursor))
            },
            Some(13) => {
                let (span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((PatternKind::Wildcard(span), cursor))
            },
            Some(n @ 14..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for StructFieldPattern {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.name.encode_impl(buffer);
        self.span.encode_impl(buffer);
        self.pattern.encode_impl(buffer);
        self.is_shorthand.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (pattern, cursor) = Pattern::decode_impl(buffer, cursor)?;
        let (is_shorthand, cursor) = bool::decode_impl(buffer, cursor)?;

        Ok((
            StructFieldPattern {
                name,
                span,
                pattern,
                is_shorthand,
            },
            cursor,
        ))
    }
}
