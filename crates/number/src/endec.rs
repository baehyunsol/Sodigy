use crate::{InternedNumber, InternedNumberValue};
use sodigy_endec::{DecodeError, Endec};

impl Endec for InternedNumber {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.value.encode_impl(buffer);
        self.is_integer.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (value, cursor) = InternedNumberValue::decode_impl(buffer, cursor)?;
        let (is_integer, cursor) = bool::decode_impl(buffer, cursor)?;
        Ok((InternedNumber { value, is_integer }, cursor))
    }
}

impl Endec for InternedNumberValue {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            InternedNumberValue::SmallInteger(n) => {
                buffer.push(0);
                n.encode_impl(buffer);
            },
            InternedNumberValue::SmallRatio { numer, denom } => {
                buffer.push(1);
                numer.encode_impl(buffer);
                denom.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (n, cursor) = i64::decode_impl(buffer, cursor + 1)?;
                Ok((InternedNumberValue::SmallInteger(n), cursor))
            },
            Some(1) => {
                let (numer, cursor) = i64::decode_impl(buffer, cursor + 1)?;
                let (denom, cursor) = u64::decode_impl(buffer, cursor)?;
                Ok((InternedNumberValue::SmallRatio { numer, denom }, cursor))
            },
            Some(n @ 2..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
