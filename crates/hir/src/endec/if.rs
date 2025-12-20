use crate::{Expr, If, Pattern};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;

impl Endec for If {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.if_span.encode_impl(buffer);
        self.cond.encode_impl(buffer);
        self.pattern.encode_impl(buffer);
        self.else_span.encode_impl(buffer);
        self.true_value.encode_impl(buffer);
        self.true_group_span.encode_impl(buffer);
        self.false_value.encode_impl(buffer);
        self.false_group_span.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (if_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (cond, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
        let (pattern, cursor) = Option::<Pattern>::decode_impl(buffer, cursor)?;
        let (else_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (true_value, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
        let (true_group_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (false_value, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
        let (false_group_span, cursor) = Span::decode_impl(buffer, cursor)?;

        Ok((
            If {
                if_span,
                cond,
                pattern,
                else_span,
                true_value,
                true_group_span,
                false_value,
                false_group_span,
            },
            cursor,
        ))
    }
}
