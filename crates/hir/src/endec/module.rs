use crate::{Module, Public};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Module {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.public.encode_impl(buffer);
        self.keyword_span.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (public, cursor) = Public::decode_impl(buffer, cursor)?;
        let (keyword_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;

        Ok((
            Module {
                public,
                keyword_span,
                name,
                name_span,
            },
            cursor,
        ))
    }
}
