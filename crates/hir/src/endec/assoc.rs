use crate::{AssociatedItem, Type};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for AssociatedItem {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.is_func.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.params.encode_impl(buffer);
        self.type_span.encode_impl(buffer);
        self.r#type.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (is_func, cursor) = bool::decode_impl(buffer, cursor)?;
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (params, cursor) = Option::<usize>::decode_impl(buffer, cursor)?;
        let (type_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (r#type, cursor) = Type::decode_impl(buffer, cursor)?;

        Ok((
            AssociatedItem {
                is_func,
                name,
                name_span,
                params,
                type_span,
                r#type,
            },
            cursor,
        ))
    }
}
