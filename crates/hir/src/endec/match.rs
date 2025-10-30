use crate::{Expr, FullPattern, Match, MatchBranch};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;

impl Endec for Match {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.keyword_span.encode_impl(buffer);
        self.value.encode_impl(buffer);
        self.branches.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (keyword_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (value, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
        let (branches, cursor) = Vec::<MatchBranch>::decode_impl(buffer, cursor)?;

        Ok((
            Match {
                keyword_span,
                value,
                branches,
            },
            cursor,
        ))
    }
}

impl Endec for MatchBranch {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.pattern.encode_impl(buffer);
        self.cond.encode_impl(buffer);
        self.value.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (pattern, cursor) = FullPattern::decode_impl(buffer, cursor)?;
        let (cond, cursor) = Option::<Expr>::decode_impl(buffer, cursor)?;
        let (value, cursor) = Expr::decode_impl(buffer, cursor)?;

        Ok((
            MatchBranch {
                pattern,
                cond,
                value,
            },
            cursor,
        ))
    }
}
