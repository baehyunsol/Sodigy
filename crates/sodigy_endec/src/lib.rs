#![deny(unused_imports)]

mod err;
mod int;
mod no_cycle;
mod session;

#[cfg(test)]
mod tests;

pub use err::EndecErr;
pub use session::EndecSession;

pub trait Endec {
    fn encode(&self, buf: &mut Vec<u8>, sess: &mut EndecSession);

    /// It moves the cursor (`ind`) after decoding. If the decoding fails, it may or may not move the cursor.
    fn decode(buf: &[u8], ind: &mut usize, sess: &mut EndecSession) -> Result<Self, EndecErr> where Self: Sized;
}

impl Endec for char {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        (*self as u32).encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        let c = u32::decode(buf, ind, session)?;

        char::from_u32(c).ok_or_else(|| EndecErr::FromUtf8Error)
    }
}

impl Endec for bool {
    fn encode(&self, buf: &mut Vec<u8>, _: &mut EndecSession) {
        buf.push(*self as u8);
    }

    fn decode(buf: &[u8], ind: &mut usize, _: &mut EndecSession) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(false),
                    1 => Ok(true),
                    2.. => Err(EndecErr::InvalidEnumVariant { variant_index: *n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}

impl Endec for String {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        // Does this clone the inner buffer?
        (<&str as Into<Vec<u8>>>::into(&self)).encode(buf, session);
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        let v8 = Vec::<u8>::decode(buf, ind, session)?;

        String::from_utf8(v8).map_err(|e| e.into())
    }
}

impl<T: Endec> Endec for Vec<T> {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.len().encode(buf, session);

        for v in self.iter() {
            v.encode(buf, session);
        }
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        let len = usize::decode(buf, ind, session)?;
        let mut result = Vec::with_capacity(len);

        for _ in 0..len {
            result.push(T::decode(buf, ind, session)?);
        }

        Ok(result)
    }
}

impl<T: Endec> Endec for Option<T> {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        if let Some(v) = self {
            buf.push(1);
            v.encode(buf, session);
        }

        else {
            buf.push(0);
        }
    }

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(None),
                    1 => Ok(Some(T::decode(buf, ind, session)?)),
                    2.. => Err(EndecErr::InvalidEnumVariant { variant_index: *n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}

impl<T: Endec, U: Endec> Endec for (T, U) {
    fn encode(&self, buf: &mut Vec<u8>, sess: &mut EndecSession) {
        self.0.encode(buf, sess);
        self.1.encode(buf, sess);
    }

    fn decode(buf: &[u8], ind: &mut usize, sess: &mut EndecSession) -> Result<Self, EndecErr> {
        Ok((
            T::decode(buf, ind, sess)?,
            U::decode(buf, ind, sess)?,
        ))
    }
}
