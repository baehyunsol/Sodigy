use crate::{Pattern, PatternKind, Type};
use sodigy_endec::{DecodeError, Endec};
use sodigy_name_analysis::IdentWithOrigin;
use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::InfixOp;

impl Endec for Pattern {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.r#type.encode_impl(buffer);
        self.kind.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (name, cursor) = Option::<InternedString>::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;
        let (r#type, cursor) = Option::<Type>::decode_impl(buffer, cursor)?;
        let (kind, cursor) = PatternKind::decode_impl(buffer, cursor)?;

        Ok((
            Pattern {
                name,
                name_span,
                r#type,
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
            PatternKind::DollarIdent(id) => {
                buffer.push(1);
                id.encode_impl(buffer);
            },
            PatternKind::Number { n, span } => {
                buffer.push(2);
                n.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            PatternKind::String { binary, s, span } => {
                buffer.push(3);
                binary.encode_impl(buffer);
                s.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            PatternKind::Regex { s, span } => {
                buffer.push(4);
                s.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            PatternKind::Char { ch, span } => {
                buffer.push(5);
                ch.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            PatternKind::Byte { b, span } => {
                buffer.push(6);
                b.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            PatternKind::Path(path) => {
                buffer.push(7);
                path.encode_impl(buffer);
            },
            PatternKind::TupleStruct { r#struct, elements, dot_dot_span, group_span } => {
                buffer.push(8);
                r#struct.encode_impl(buffer);
                elements.encode_impl(buffer);
                dot_dot_span.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            PatternKind::Tuple { elements, dot_dot_span, group_span } => {
                buffer.push(9);
                elements.encode_impl(buffer);
                dot_dot_span.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            PatternKind::List { elements, dot_dot_span, group_span } => {
                buffer.push(10);
                elements.encode_impl(buffer);
                dot_dot_span.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            PatternKind::Range { lhs, rhs, op_span, is_inclusive } => {
                buffer.push(11);
                lhs.encode_impl(buffer);
                rhs.encode_impl(buffer);
                op_span.encode_impl(buffer);
                is_inclusive.encode_impl(buffer);
            },
            PatternKind::InfixOp { op, lhs, rhs, op_span } => {
                buffer.push(12);
                op.encode_impl(buffer);
                lhs.encode_impl(buffer);
                rhs.encode_impl(buffer);
                op_span.encode_impl(buffer);
            },
            PatternKind::Or { lhs, rhs, op_span } => {
                buffer.push(13);
                lhs.encode_impl(buffer);
                rhs.encode_impl(buffer);
                op_span.encode_impl(buffer);
            },
            PatternKind::Wildcard(span) => {
                buffer.push(14);
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
                let (id, cursor) = IdentWithOrigin::decode_impl(buffer, cursor + 1)?;
                Ok((PatternKind::DollarIdent(id), cursor))
            },
            Some(2) => {
                let (n, cursor) = InternedNumber::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Number { n, span }, cursor))
            },
            Some(3) => {
                let (binary, cursor) = bool::decode_impl(buffer, cursor + 1)?;
                let (s, cursor) = InternedString::decode_impl(buffer, cursor)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::String { binary, s, span }, cursor))
            },
            Some(4) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Regex { s, span }, cursor))
            },
            Some(5) => {
                let (ch, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Char { ch, span }, cursor))
            },
            Some(6) => {
                let (b, cursor) = u8::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Byte { b, span }, cursor))
            },
            Some(7) => {
                let (path, cursor) = Vec::<(InternedString, Span)>::decode_impl(buffer, cursor + 1)?;
                Ok((PatternKind::Path(path), cursor))
            },
            Some(8) => {
                let (r#struct, cursor) = Vec::<(InternedString, Span)>::decode_impl(buffer, cursor + 1)?;
                let (elements, cursor) = Vec::<Pattern>::decode_impl(buffer, cursor + 1)?;
                let (dot_dot_span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::TupleStruct { r#struct, elements, dot_dot_span, group_span }, cursor))
            },
            Some(9) => {
                let (elements, cursor) = Vec::<Pattern>::decode_impl(buffer, cursor + 1)?;
                let (dot_dot_span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Tuple { elements, dot_dot_span, group_span }, cursor))
            },
            Some(10) => {
                let (elements, cursor) = Vec::<Pattern>::decode_impl(buffer, cursor + 1)?;
                let (dot_dot_span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::List { elements, dot_dot_span, group_span }, cursor))
            },
            Some(11) => {
                let (lhs, cursor) = Option::<Box<Pattern>>::decode_impl(buffer, cursor + 1)?;
                let (rhs, cursor) = Option::<Box<Pattern>>::decode_impl(buffer, cursor)?;
                let (op_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (is_inclusive, cursor) = bool::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Range { lhs, rhs, op_span, is_inclusive }, cursor))
            },
            Some(12) => {
                let (op, cursor) = InfixOp::decode_impl(buffer, cursor + 1)?;
                let (lhs, cursor) = Box::<Pattern>::decode_impl(buffer, cursor)?;
                let (rhs, cursor) = Box::<Pattern>::decode_impl(buffer, cursor)?;
                let (op_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::InfixOp { op, lhs, rhs, op_span }, cursor))
            },
            Some(13) => {
                let (lhs, cursor) = Box::<Pattern>::decode_impl(buffer, cursor + 1)?;
                let (rhs, cursor) = Box::<Pattern>::decode_impl(buffer, cursor)?;
                let (op_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((PatternKind::Or { lhs, rhs, op_span }, cursor))
            },
            Some(14) => {
                let (span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((PatternKind::Wildcard(span), cursor))
            },
            Some(n @ 15..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
