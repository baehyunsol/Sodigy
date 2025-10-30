use crate::{DecodeError, Endec};

impl<T: Endec> Endec for Option<T> {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            None => {
                buffer.push(0);
            },
            Some(v) => {
                buffer.push(1);
                v.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((None, cursor + 1)),
            Some(1) => {
                let (v, cursor) = T::decode_impl(buffer, cursor + 1)?;
                Ok((Some(v), cursor))
            },
            Some(n) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl <T1: Endec, T2: Endec> Endec for (T1, T2) {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.0.encode_impl(buffer);
        self.1.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (e1, cursor) = T1::decode_impl(buffer, cursor)?;
        let (e2, cursor) = T2::decode_impl(buffer, cursor)?;
        Ok(((e1, e2), cursor))
    }
}
