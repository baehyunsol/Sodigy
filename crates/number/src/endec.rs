use crate::{BigInt, InternedNumber, Ratio};
use sodigy_endec::{DecodeError, Endec};

impl Endec for InternedNumber {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.0.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (n, cursor) = u128::decode_impl(buffer, cursor)?;
        Ok((InternedNumber(n), cursor))
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
