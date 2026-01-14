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
            Some(n @ 2..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl<T: Endec, E: Endec> Endec for Result<T, E> {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Ok(v) => {
                buffer.push(0);
                v.encode_impl(buffer);
            },
            Err(e) => {
                buffer.push(1);
                e.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (v, cursor) = T::decode_impl(buffer, cursor + 1)?;
                Ok((Ok(v), cursor))
            },
            Some(1) => {
                let (e, cursor) = E::decode_impl(buffer, cursor + 1)?;
                Ok((Err(e), cursor))
            },
            Some(n @ 2..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for () {
    fn encode_impl(&self, _: &mut Vec<u8>) {
        // ZST
    }

    fn decode_impl(_: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        Ok(((), cursor))
    }
}

impl <T1: Endec> Endec for (T1,) {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.0.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (e, cursor) = T1::decode_impl(buffer, cursor)?;
        Ok(((e,), cursor))
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

impl <T1: Endec, T2: Endec, T3: Endec> Endec for (T1, T2, T3) {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.0.encode_impl(buffer);
        self.1.encode_impl(buffer);
        self.2.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (e1, cursor) = T1::decode_impl(buffer, cursor)?;
        let (e2, cursor) = T2::decode_impl(buffer, cursor)?;
        let (e3, cursor) = T3::decode_impl(buffer, cursor)?;
        Ok(((e1, e2, e3), cursor))
    }
}
