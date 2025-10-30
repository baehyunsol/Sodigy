use crate::{FullPattern, Pattern, Type};
use sodigy_endec::{DecodeError, Endec};
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
        todo!()
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        todo!()
    }
}
