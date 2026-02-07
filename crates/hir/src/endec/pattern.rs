use crate::{Path, Pattern, PatternKind, StructFieldPattern};
use sodigy_endec::{DecodeError, Endec};
use sodigy_parse::RestPattern;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::Constant;

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
            PatternKind::Path(path) => {
                buffer.push(0);
                path.encode_impl(buffer);
            },
            PatternKind::Constant(constant) => {
                buffer.push(1);
                constant.encode_impl(buffer);
            },
            PatternKind::NameBinding { id, span } => {
                buffer.push(2);
                id.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            PatternKind::Regex { s, span } => {
                buffer.push(3);
                s.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            PatternKind::Struct { r#struct, fields, rest, group_span } => {
                buffer.push(4);
                r#struct.encode_impl(buffer);
                fields.encode_impl(buffer);
                rest.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            PatternKind::TupleStruct { r#struct, elements, rest, group_span } => {
                buffer.push(5);
                r#struct.encode_impl(buffer);
                elements.encode_impl(buffer);
                rest.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            PatternKind::Tuple { elements, rest, group_span } => {
                buffer.push(6);
                elements.encode_impl(buffer);
                rest.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            PatternKind::List { elements, rest, group_span } => {
                buffer.push(7);
                elements.encode_impl(buffer);
                rest.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            PatternKind::Range { lhs, rhs, op_span, is_inclusive } => {
                buffer.push(8);
                lhs.encode_impl(buffer);
                rhs.encode_impl(buffer);
                op_span.encode_impl(buffer);
                is_inclusive.encode_impl(buffer);
            },
            PatternKind::Or { lhs, rhs, op_span } => {
                buffer.push(9);
                lhs.encode_impl(buffer);
                rhs.encode_impl(buffer);
                op_span.encode_impl(buffer);
            },
            PatternKind::Wildcard(span) => {
                buffer.push(10);
                span.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (path, cursor) = Path::decode_impl(buffer, cursor + 1)?;
                Ok((PatternKind::Path(path), cursor))
            },
            Some(1) => {
                let (constant, cursor) = Constant::decode_impl(buffer, cursor + 1)?;
                Ok((PatternKind::Constant(constant), cursor))
            },
            Some(2) => {
                let (id, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::NameBinding { id, span }, cursor))
            },
            Some(3) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Regex { s, span }, cursor))
            },
            Some(4) => {
                let (r#struct, cursor) = Path::decode_impl(buffer, cursor + 1)?;
                let (fields, cursor) = Vec::<StructFieldPattern>::decode_impl(buffer, cursor)?;
                let (rest, cursor) = Option::<RestPattern>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Struct { r#struct, fields, rest, group_span }, cursor))
            },
            Some(5) => {
                let (r#struct, cursor) = Path::decode_impl(buffer, cursor + 1)?;
                let (elements, cursor) = Vec::<Pattern>::decode_impl(buffer, cursor)?;
                let (rest, cursor) = Option::<RestPattern>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::TupleStruct { r#struct, elements, rest, group_span }, cursor))
            },
            Some(6) => {
                let (elements, cursor) = Vec::<Pattern>::decode_impl(buffer, cursor + 1)?;
                let (rest, cursor) = Option::<RestPattern>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Tuple { elements, rest, group_span }, cursor))
            },
            Some(7) => {
                let (elements, cursor) = Vec::<Pattern>::decode_impl(buffer, cursor + 1)?;
                let (rest, cursor) = Option::<RestPattern>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::List { elements, rest, group_span }, cursor))
            },
            Some(8) => {
                let (lhs, cursor) = Option::<Box<Pattern>>::decode_impl(buffer, cursor + 1)?;
                let (rhs, cursor) = Option::<Box<Pattern>>::decode_impl(buffer, cursor)?;
                let (op_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (is_inclusive, cursor) = bool::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Range { lhs, rhs, op_span, is_inclusive }, cursor))
            },
            Some(9) => {
                let (lhs, cursor) = Box::<Pattern>::decode_impl(buffer, cursor + 1)?;
                let (rhs, cursor) = Box::<Pattern>::decode_impl(buffer, cursor)?;
                let (op_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Or { lhs, rhs, op_span }, cursor))
            },
            Some(10) => {
                let (span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((PatternKind::Wildcard(span), cursor))
            },
            Some(n @ 11..) => Err(DecodeError::InvalidEnumVariant(*n)),
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
