use crate::{Bytecode, Func};
use sodigy_endec::{DecodeError, Endec};
use sodigy_error::FuncEffect;
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Func {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.effect.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.params.encode_impl(buffer);
        self.bytecodes.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (effect, cursor) = FuncEffect::decode_impl(buffer, cursor)?;
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (params, cursor) = usize::decode_impl(buffer, cursor)?;
        let (bytecodes, cursor) = Vec::<Bytecode>::decode_impl(buffer, cursor)?;

        Ok((Func { effect, name, name_span, params, bytecodes }, cursor))
    }
}
