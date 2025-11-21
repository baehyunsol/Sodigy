use crate::{Assert, Expr};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Assert {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.name.encode_impl(buffer);
        self.keyword_span.encode_impl(buffer);
        self.always.encode_impl(buffer);
        self.note.encode_impl(buffer);
        self.note_decorator_span.encode_impl(buffer);
        self.value.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (name, cursor) = Option::<InternedString>::decode_impl(buffer, cursor)?;
        let (keyword_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (always, cursor) = bool::decode_impl(buffer, cursor)?;
        let (note, cursor) = Option::<Expr>::decode_impl(buffer, cursor)?;
        let (note_decorator_span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;
        let (value, cursor) = Expr::decode_impl(buffer, cursor)?;

        Ok((
            Assert {
                name,
                keyword_span,
                always,
                note,
                note_decorator_span,
                value,
            },
            cursor,
        ))
    }
}
