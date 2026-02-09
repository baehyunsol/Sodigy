use crate::Value;
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;

impl Endec for Value {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Value::Scalar(v) => {
                buffer.push(0);
                v.encode_impl(buffer);
            },
            Value::Compound(vs) => {
                buffer.push(1);
                vs.encode_impl(buffer);
            },
            Value::FuncPointer { def_span, program_counter } => {
                buffer.push(2);
                def_span.encode_impl(buffer);
                program_counter.encode_impl(buffer);
            },
            Value::Span(span) => {
                buffer.push(3);
                span.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (v, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                Ok((Value::Scalar(v), cursor))
            },
            Some(1) => {
                let (vs, cursor) = Vec::<Value>::decode_impl(buffer, cursor + 1)?;
                Ok((Value::Compound(vs), cursor))
            },
            Some(2) => {
                let (def_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                let (program_counter, cursor) = Option::<usize>::decode_impl(buffer, cursor)?;
                Ok((Value::FuncPointer { def_span, program_counter }, cursor))
            },
            Some(3) => {
                let (span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Value::Span(span), cursor))
            },
            Some(n @ 4..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
