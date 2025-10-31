use crate::{BigInt, InternedNumber, InternedNumberValue, Ratio};
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
            InternedNumberValue::SmallInt(n) => {
                buffer.push(0);
                n.encode_impl(buffer);
            },
            InternedNumberValue::SmallRatio { numer, denom } => {
                buffer.push(1);
                numer.encode_impl(buffer);
                denom.encode_impl(buffer);
            },
            InternedNumberValue::BigInt(n) => {
                buffer.push(2);
                n.encode_impl(buffer);
            },
            InternedNumberValue::BigRatio(n) => {
                buffer.push(3);
                n.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (n, cursor) = i64::decode_impl(buffer, cursor + 1)?;
                Ok((InternedNumberValue::SmallInt(n), cursor))
            },
            Some(1) => {
                let (numer, cursor) = i64::decode_impl(buffer, cursor + 1)?;
                let (denom, cursor) = u64::decode_impl(buffer, cursor)?;
                Ok((InternedNumberValue::SmallRatio { numer, denom }, cursor))
            },
            Some(2) => {
                let (n, cursor) = BigInt::decode_impl(buffer, cursor + 1)?;
                Ok((InternedNumberValue::BigInt(n), cursor))
            },
            Some(3) => {
                let (n, cursor) = Ratio::decode_impl(buffer, cursor + 1)?;
                Ok((InternedNumberValue::BigRatio(n), cursor))
            },
            Some(n @ 4..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for BigInt {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.is_neg.encode_impl(buffer);
        self.nums.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (is_neg, cursor) = bool::decode_impl(buffer, cursor)?;
        let (nums, cursor) = Vec::<u32>::decode_impl(buffer, cursor)?;
        Ok((BigInt { is_neg, nums }, cursor))
    }
}

impl Endec for Ratio {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.numer.encode_impl(buffer);
        self.denom.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (numer, cursor) = BigInt::decode_impl(buffer, cursor)?;
        let (denom, cursor) = BigInt::decode_impl(buffer, cursor)?;
        Ok((Ratio { numer, denom }, cursor))
    }
}
