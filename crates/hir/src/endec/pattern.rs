use crate::{FullPattern, Pattern, Type};
use sodigy_endec::{DecodeError, Endec};
use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for FullPattern {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.r#type.encode_impl(buffer);
        self.pattern.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (name, cursor) = Option::<InternedString>::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;
        let (r#type, cursor) = Option::<Type>::decode_impl(buffer, cursor)?;
        let (pattern, cursor) = Pattern::decode_impl(buffer, cursor)?;

        Ok((
            FullPattern {
                name,
                name_span,
                r#type,
                pattern,
            },
            cursor,
        ))
    }
}

impl Endec for Pattern {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Pattern::Number { n, span } => {
                buffer.push(0);
                n.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Pattern::Identifier { id, span } => {
                buffer.push(1);
                id.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Pattern::Wildcard(span) => {
                buffer.push(2);
                span.encode_impl(buffer);
            },
            Pattern::Tuple { elements, group_span } => {
                buffer.push(3);
                elements.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            Pattern::List { elements, group_span } => {
                buffer.push(4);
                elements.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            Pattern::Range { lhs, rhs, op_span, is_inclusive } => {
                buffer.push(5);
                lhs.encode_impl(buffer);
                rhs.encode_impl(buffer);
                op_span.encode_impl(buffer);
                is_inclusive.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (n, cursor) = InternedNumber::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Pattern::Number { n, span }, cursor))
            },
            Some(1) => {
                let (id, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Pattern::Identifier { id, span }, cursor))
            },
            Some(2) => {
                let (span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Pattern::Wildcard(span), cursor))
            },
            Some(3) => {
                let (elements, cursor) = Vec::<FullPattern>::decode_impl(buffer, cursor + 1)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Pattern::Tuple { elements, group_span }, cursor))
            },
            Some(4) => {
                let (elements, cursor) = Vec::<FullPattern>::decode_impl(buffer, cursor + 1)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Pattern::List { elements, group_span }, cursor))
            },
            Some(5) => {
                let (lhs, cursor) = Option::<Box<Pattern>>::decode_impl(buffer, cursor + 1)?;
                let (rhs, cursor) = Option::<Box<Pattern>>::decode_impl(buffer, cursor)?;
                let (op_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (is_inclusive, cursor) = bool::decode_impl(buffer, cursor)?;
                Ok((Pattern::Range { lhs, rhs, op_span, is_inclusive }, cursor))
            },
            Some(n) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
