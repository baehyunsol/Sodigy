use crate::Visibility;
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;

impl Endec for Visibility {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.keyword_span.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (keyword_span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;
        Ok((Visibility { keyword_span }, cursor))
    }
}
