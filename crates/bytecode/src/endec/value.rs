use crate::Value;
use sodigy_endec::{DecodeError, Endec};
use sodigy_number::BigInt;
use sodigy_span::Span;

impl Endec for Value {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Value::Scalar(v) => {
                buffer.push(0);
                v.encode_impl(buffer);
            },
            Value::Int(n) => {
                buffer.push(1);
                n.encode_impl(buffer);
            },
            Value::List(es) => {
                buffer.push(2);
                es.encode_impl(buffer);
            },
            Value::Compound(vs) => {
                buffer.push(3);
                vs.encode_impl(buffer);
            },
            Value::FuncPointer { def_span, program_counter } => {
                buffer.push(4);
                def_span.encode_impl(buffer);
                program_counter.encode_impl(buffer);
            },
            Value::Span(span) => {
                buffer.push(5);
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
                let (n, cursor) = BigInt::decode_impl(buffer, cursor + 1)?;
                Ok((Value::Int(n), cursor))
            },
            Some(2) => {
                let (vs, cursor) = Vec::<Value>::decode_impl(buffer, cursor + 1)?;
                Ok((Value::List(vs), cursor))
            },
            Some(3) => {
                let (vs, cursor) = Vec::<Value>::decode_impl(buffer, cursor + 1)?;
                Ok((Value::Compound(vs), cursor))
            },
            Some(4) => {
                let (def_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                let (program_counter, cursor) = Option::<usize>::decode_impl(buffer, cursor)?;
                Ok((Value::FuncPointer { def_span, program_counter }, cursor))
            },
            Some(5) => {
                let (span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Value::Span(span), cursor))
            },
            Some(n @ 6..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
