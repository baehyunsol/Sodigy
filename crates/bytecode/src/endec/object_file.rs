use crate::{
    Bytecode,
    CodeKind,
    CodeSection,
    ExprHash,
    ObjectFile,
    Value,
};
use sodigy_endec::{DecodeError, Endec};
use sodigy_error::FuncEffect;
use sodigy_span::{Span, SpanHash};

impl Endec for ObjectFile {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.data.encode_impl(buffer);
        self.code.encode_impl(buffer);
        self.main_entry.encode_impl(buffer);
        self.asserts.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (data, cursor) = Vec::<(ExprHash, Value)>::decode_impl(buffer, cursor)?;
        let (code, cursor) = Vec::<CodeSection>::decode_impl(buffer, cursor)?;
        let (main_entry, cursor) = Option::<SpanHash>::decode_impl(buffer, cursor)?;
        let (asserts, cursor) = Vec::<SpanHash>::decode_impl(buffer, cursor)?;

        Ok((ObjectFile { data, code, main_entry, asserts }, cursor))
    }
}

impl Endec for CodeSection {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.label.encode_impl(buffer);
        self.span.encode_impl(buffer);
        self.kind.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.params.encode_impl(buffer);
        self.effect.encode_impl(buffer);
        self.code.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (label, cursor) = SpanHash::decode_impl(buffer, cursor)?;
        let (span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;
        let (kind, cursor) = CodeKind::decode_impl(buffer, cursor)?;
        let (name, cursor) = String::decode_impl(buffer, cursor)?;
        let (params, cursor) = Option::<usize>::decode_impl(buffer, cursor)?;
        let (effect, cursor) = FuncEffect::decode_impl(buffer, cursor)?;
        let (code, cursor) = Vec::<Bytecode>::decode_impl(buffer, cursor)?;

        Ok((CodeSection { label, span, kind, name, params, effect, code }, cursor))
    }
}
