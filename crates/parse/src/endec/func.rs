use crate::{Expr, FuncArg};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for FuncArg {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.keyword.encode_impl(buffer);
        self.arg.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (keyword, cursor) = Option::<(InternedString, Span)>::decode_impl(buffer, cursor)?;
        let (arg, cursor) = Expr::decode_impl(buffer, cursor)?;

        Ok((
            FuncArg {
                keyword,
                arg,
            },
            cursor,
        ))
    }
}
