use crate::{Expr, Match, MatchArm, Pattern};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;

impl Endec for Match {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.keyword_span.encode_impl(buffer);
        self.scrutinee.encode_impl(buffer);
        self.arms.encode_impl(buffer);
        self.group_span.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (keyword_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (scrutinee, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
        let (arms, cursor) = Vec::<MatchArm>::decode_impl(buffer, cursor)?;
        let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;

        Ok((
            Match {
                keyword_span,
                scrutinee,
                arms,
                group_span,
            },
            cursor,
        ))
    }
}

impl Endec for MatchArm {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.pattern.encode_impl(buffer);
        self.guard.encode_impl(buffer);
        self.value.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (pattern, cursor) = Pattern::decode_impl(buffer, cursor)?;
        let (guard, cursor) = Option::<Expr>::decode_impl(buffer, cursor)?;
        let (value, cursor) = Expr::decode_impl(buffer, cursor)?;

        Ok((
            MatchArm {
                pattern,
                guard,
                value,
            },
            cursor,
        ))
    }
}
