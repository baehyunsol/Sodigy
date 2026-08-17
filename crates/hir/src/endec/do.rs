use crate::{Do, Expr};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;

impl Endec for Do {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.keyword_span.encode_impl(buffer);
        self.value.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (keyword_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (value, cursor) = Expr::decode_impl(buffer, cursor)?;

        Ok((
            Do {
                keyword_span,
                value,
            },
            cursor,
        ))
    }
}
