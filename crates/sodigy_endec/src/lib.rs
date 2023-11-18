#![deny(unused_imports)]

mod err;
mod int;

#[cfg(test)]
mod tests;

pub use err::EndecErr;

pub trait Endec {
    fn encode(&self, buf: &mut Vec<u8>);

    /// It moves the cursor (`ind`) after decoding. If the decoding fails, it may or may not move the cursor.
    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> where Self: Sized;
}

impl Endec for char {
    fn encode(&self, buf: &mut Vec<u8>) {
        (*self as u32).encode(buf);
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        let c = u32::decode(buf, ind)?;

        char::from_u32(c).ok_or_else(|| EndecErr::FromUtf8Error)
    }
}

impl Endec for bool {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(*self as u8);
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(false),
                    1 => Ok(true),
                    n => Err(EndecErr::InvalidEnumVariant { variant_index: n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}

impl Endec for String {
    fn encode(&self, buf: &mut Vec<u8>) {
        // Does this clone the inner buffer?
        (<&str as Into<Vec<u8>>>::into(&self)).encode(buf);
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        let v8 = Vec::<u8>::decode(buf, ind)?;

        String::from_utf8(v8).map_err(|e| e.into())
    }
}

impl<T: Endec> Endec for Vec<T> {
    fn encode(&self, buf: &mut Vec<u8>) {
        self.len().encode(buf);

        for v in self.iter() {
            v.encode(buf);
        }
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        let len = usize::decode(buf, ind)?;
        let mut result = Vec::with_capacity(len);

        for _ in 0..len {
            result.push(T::decode(buf, ind)?);
        }

        Ok(result)
    }
}

impl<T: Endec> Endec for Option<T> {
    fn encode(&self, buf: &mut Vec<u8>) {
        if let Some(v) = self {
            buf.push(1);
            v.encode(buf);
        }

        else {
            buf.push(0);
        }
    }

    fn decode(buf: &[u8], ind: &mut usize) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(None),
                    1 => Ok(Some(T::decode(buf, ind)?)),
                    n => Err(EndecErr::InvalidEnumVariant { variant_index: n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}
