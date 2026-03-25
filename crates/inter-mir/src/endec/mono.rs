use crate::Monomorphization;
use sodigy_endec::{DecodeError, Endec};
use sodigy_mir::Type;
use sodigy_span::Span;
use std::collections::HashMap;

impl Endec for Monomorphization {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.id.encode_impl(buffer);
        self.def_span.encode_impl(buffer);
        self.call_span.encode_impl(buffer);
        self.generics.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (id, cursor) = u64::decode_impl(buffer, cursor)?;
        let (def_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (call_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (generics, cursor) = HashMap::<Span, Type>::decode_impl(buffer, cursor)?;

        Ok((
            Monomorphization {
                def_span,
                call_span,
                generics,
                id,
            },
            cursor,
        ))
    }
}
