use crate::RestPattern;
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for RestPattern {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.span.encode_impl(buffer);
        self.index.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (index, cursor) = usize::decode_impl(buffer, cursor)?;
        let (name, cursor) = Option::<InternedString>::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;

        Ok((
            RestPattern {
                span,
                index,
                name,
                name_span,
            },
            cursor,
        ))
    }
}
