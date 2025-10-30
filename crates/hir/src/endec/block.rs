use crate::{Assert, Block, Expr, Let, Use};
use sodigy_endec::{DecodeError, Endec};
use sodigy_name_analysis::UseCount;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

impl Endec for Block {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.group_span.encode_impl(buffer);
        self.lets.encode_impl(buffer);
        self.asserts.encode_impl(buffer);
        self.uses.encode_impl(buffer);
        self.value.encode_impl(buffer);
        self.use_counts.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (lets, cursor) = Vec::<Let>::decode_impl(buffer, cursor)?;
        let (asserts, cursor) = Vec::<Assert>::decode_impl(buffer, cursor)?;
        let (uses, cursor) = Vec::<Use>::decode_impl(buffer, cursor)?;
        let (value, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
        let (use_counts, cursor) = HashMap::<InternedString, UseCount>::decode_impl(buffer, cursor)?;

        Ok((
            Block {
                group_span,
                lets,
                asserts,
                uses,
                value,
                use_counts,
            },
            cursor,
        ))
    }
}
