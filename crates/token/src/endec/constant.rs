use crate::Constant;
use sodigy_endec::{DecodeError, Endec};
use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Constant {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Constant::Number { n, span } => {
                buffer.push(0);
                n.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Constant::String { binary, s, span } => {
                buffer.push(1);
                binary.encode_impl(buffer);
                s.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Constant::Char { ch, span } => {
                buffer.push(2);
                ch.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Constant::Byte { b, span } => {
                buffer.push(3);
                b.encode_impl(buffer);
                span.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (n, cursor) = InternedNumber::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Constant::Number { n, span }, cursor))
            },
            Some(1) => {
                let (binary, cursor) = bool::decode_impl(buffer, cursor + 1)?;
                let (s, cursor) = InternedString::decode_impl(buffer, cursor)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Constant::String { binary, s, span }, cursor))
            },
            Some(2) => {
                let (ch, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Constant::Char { ch, span }, cursor))
            },
            Some(3) => {
                let (b, cursor) = u8::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Constant::Byte { b, span }, cursor))
            },
            Some(n @ 4..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
