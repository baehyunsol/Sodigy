use crate::{DecodeError, Endec};

mod adt;
mod collections;
mod int;

impl Endec for bool {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        buffer.push(if *self { 1 } else { 0 });
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((false, cursor + 1)),
            Some(1) => Ok((true, cursor + 1)),
            Some(n) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl<T: Endec> Endec for Box<T> {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.as_ref().encode_impl(buffer)
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (v, cursor) = T::decode_impl(buffer, cursor)?;
        Ok((Box::new(v), cursor))
    }
}

impl Endec for String {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.len().encode_impl(buffer);
        buffer.extend(self.as_bytes());
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (v, cursor) = Vec::<u8>::decode_impl(buffer, cursor)?;
        Ok((String::from_utf8(v).map_err(|_| DecodeError::InvalidUtf8)?, cursor))
    }
}
