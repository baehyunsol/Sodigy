use crate::{Endec, EndecError, EndecSession};

impl Endec for u8 {
    fn encode(&self, buf: &mut Vec<u8>, _: &mut EndecSession) {
        buf.push(*self);
    }

    fn decode(buf: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        if let Some(n) = buf.get(*index) {
            *index += 1;
            return Ok(*n);
        }

        else {
            return Err(EndecError::eof());
        }
    }
}

impl Endec for u16 {
    fn encode(&self, buf: &mut Vec<u8>, _: &mut EndecSession) {
        let hi = (*self >> 8) as u8;
        let lo = (*self & 0xff) as u8;

        buf.push(hi);
        buf.push(lo);
    }

    fn decode(buf: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match (buf.get(*index), buf.get(*index + 1)) {
            (Some(m), Some(n)) => {
                *index += 2;

                Ok(((*m as u16) << 8) | *n as u16)
            },
            _ => Err(EndecError::eof()),
        }
    }
}

impl Endec for u32 {
    fn encode(&self, buf: &mut Vec<u8>, _: &mut EndecSession) {
        if *self < (1 << 14) {
            if *self < (1 << 7) {
                buf.push(*self as u8);
            }

            else {
                buf.push((*self >> 7) as u8 | (1 << 7));
                buf.push((*self & 0x7f) as u8);
            }
        }

        else {
            if *self < (1 << 21) {
                buf.push((*self >> 14) as u8 | (1 << 7));
                buf.push(((*self >> 7) & 0x7f) as u8 | (1 << 7));
                buf.push((*self & 0x7f) as u8);
            }

            else if *self < (1 << 28) {
                buf.push((*self >> 21) as u8 | (1 << 7));
                buf.push(((*self >> 14) & 0x7f) as u8 | (1 << 7));
                buf.push(((*self >> 7) & 0x7f) as u8 | (1 << 7));
                buf.push((*self & 0x7f) as u8);
            }

            else {
                buf.push((*self >> 28) as u8 | (1 << 7));
                buf.push(((*self >> 21) & 0x7f) as u8 | (1 << 7));
                buf.push(((*self >> 14) & 0x7f) as u8 | (1 << 7));
                buf.push(((*self >> 7) & 0x7f) as u8 | (1 << 7));
                buf.push((*self & 0x7f) as u8);
            }
        }
    }

    fn decode(buf: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        let mut result: u32 = 0;

        loop {
            if let Some(n) = buf.get(*index) {
                *index += 1;

                if *n < (1 << 7) {
                    result = result.checked_shl(7).ok_or(EndecError::overflow())?;
                    result = result.checked_add(*n as u32).ok_or(EndecError::overflow())?;
                    return Ok(result);
                }

                else {
                    result = result.checked_shl(7).ok_or(EndecError::overflow())?;
                    result = result.checked_add((*n - (1 << 7)) as u32).ok_or(EndecError::overflow())?;
                }
            }

            else {
                return Err(EndecError::eof());
            }
        }
    }
}

impl Endec for u64 {
    fn encode(&self, buf: &mut Vec<u8>, _: &mut EndecSession) {
        if *self < (1 << 28) {
            if *self < (1 << 14) {
                if *self < (1 << 7) {
                    buf.push(*self as u8);
                }

                else {
                    buf.push((*self >> 7) as u8 | (1 << 7));
                    buf.push((*self & 0x7f) as u8);
                }
            }

            else {
                if *self < (1 << 21) {
                    buf.push((*self >> 14) as u8 | (1 << 7));
                    buf.push(((*self >> 7) & 0x7f) as u8 | (1 << 7));
                    buf.push((*self & 0x7f) as u8);
                }

                else {
                    buf.push((*self >> 21) as u8 | (1 << 7));
                    buf.push(((*self >> 14) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 7) & 0x7f) as u8 | (1 << 7));
                    buf.push((*self & 0x7f) as u8);
                }
            }
        }

        else {
            if *self < (1 << 42) {
                if *self < (1 << 35) {
                    buf.push((*self >> 28) as u8 | (1 << 7));
                    buf.push(((*self >> 21) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 14) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 7) & 0x7f) as u8 | (1 << 7));
                    buf.push((*self & 0x7f) as u8);
                }

                else {
                    buf.push((*self >> 35) as u8 | (1 << 7));
                    buf.push(((*self >> 28) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 21) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 14) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 7) & 0x7f) as u8 | (1 << 7));
                    buf.push((*self & 0x7f) as u8);
                }
            }

            else if *self < (1 << 56) {
                if *self < (1 << 49) {
                    buf.push((*self >> 42) as u8 | (1 << 7));
                    buf.push(((*self >> 35) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 28) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 21) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 14) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 7) & 0x7f) as u8 | (1 << 7));
                    buf.push((*self & 0x7f) as u8);
                }

                else {
                    buf.push((*self >> 49) as u8 | (1 << 7));
                    buf.push(((*self >> 42) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 35) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 28) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 21) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 14) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 7) & 0x7f) as u8 | (1 << 7));
                    buf.push((*self & 0x7f) as u8);
                }
            }

            else {
                if *self < (1 << 63) {
                    buf.push((*self >> 56) as u8 | (1 << 7));
                    buf.push(((*self >> 49) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 42) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 35) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 28) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 21) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 14) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 7) & 0x7f) as u8 | (1 << 7));
                    buf.push((*self & 0x7f) as u8);
                }

                else {
                    buf.push((*self >> 63) as u8 | (1 << 7));
                    buf.push(((*self >> 56) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 49) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 42) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 35) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 28) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 21) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 14) & 0x7f) as u8 | (1 << 7));
                    buf.push(((*self >> 7) & 0x7f) as u8 | (1 << 7));
                    buf.push((*self & 0x7f) as u8);
                }
            }
        }
    }

    // How do I make macro for this?
    fn decode(buf: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        let mut result: u64 = 0;

        loop {
            if let Some(n) = buf.get(*index) {
                *index += 1;

                if *n < (1 << 7) {
                    result = result.checked_shl(7).ok_or_else(|| EndecError::overflow())?;
                    result = result.checked_add(*n as u64).ok_or_else(|| EndecError::overflow())?;
                    return Ok(result);
                }

                else {
                    result = result.checked_shl(7).ok_or_else(|| EndecError::overflow())?;
                    result = result.checked_add((*n - (1 << 7)) as u64).ok_or_else(|| EndecError::overflow())?;
                }
            }

            else {
                return Err(EndecError::eof());
            }
        }
    }
}

impl Endec for u128 {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        let hi = (*self >> 64) as u64;
        let lo = (*self & 0xffff_ffff_ffff_ffff) as u64;

        hi.encode(buf, session);
        lo.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        let hi = u64::decode(buf, index, session)?;
        let lo = u64::decode(buf, index, session)?;

        Ok(((hi as u128) << 64) | lo as u128)
    }
}

impl Endec for usize {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        (*self as u64).encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        u64::decode(buf, index, session).map(|n| n as usize)
    }
}

macro_rules! endec_signed {
    ($ity: ty, $uty: ty) => {
        impl Endec for $ity {
            fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
                unsafe {
                    let s: $uty = std::mem::transmute(*self);
                    s.encode(buf, session);
                }
            }

            fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
                unsafe {
                    let s = <$uty>::decode(buf, index, session)?;
                    Ok(std::mem::transmute(s))
                }
            }
        }
    }
}

endec_signed!(i8, u8);
endec_signed!(i16, u16);
endec_signed!(i32, u32);
endec_signed!(i64, u64);
endec_signed!(i128, u128);
endec_signed!(isize, usize);
