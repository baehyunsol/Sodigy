use crate::Poly;
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Poly {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.decorator_span.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.has_default_impl.encode_impl(buffer);
        self.impls.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (decorator_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (has_default_impl, cursor) = bool::decode_impl(buffer, cursor)?;
        let (impls, cursor) = Vec::<Span>::decode_impl(buffer, cursor)?;

        Ok((
            Poly {
                decorator_span,
                name,
                name_span,
                has_default_impl,
                impls,
            },
            cursor,
        ))
    }
}
