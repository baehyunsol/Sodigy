use super::{Endec, EndecError};

impl Endec for u8 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.push(*self);
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        if let Some(n) = buffer.get(*index) {
            *index += 1;
            Ok(*n)
        } else {
            Err(EndecError::UnexpectedEof)
        }
    }
}

impl Endec for u16 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let a = (*self / 256) as u8;
        let b = (*self % 256) as u8;
        buffer.push(a);
        buffer.push(b);
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        if *index + 1 > buffer.len() {
            Err(EndecError::UnexpectedEof)
        } else {
            let a = buffer[*index];
            let b = buffer[*index + 1];
            *index += 2;

            Ok(a as u16 * 256 | b as u16)
        }
    }
}

impl Endec for u32 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let a = (*self / 16777216) as u8;
        let b = (*self / 65536 % 256) as u8;
        let c = (*self / 256 % 256) as u8;
        let d = (*self % 256) as u8;
        buffer.push(a);
        buffer.push(b);
        buffer.push(c);
        buffer.push(d);
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        if *index + 3 > buffer.len() {
            Err(EndecError::UnexpectedEof)
        } else {
            let a = buffer[*index];
            let b = buffer[*index + 1];
            let c = buffer[*index + 2];
            let d = buffer[*index + 3];
            *index += 4;

            Ok(a as u32 * 16777216 | b as u32 * 65536 | c as u32 * 256 | d as u32)
        }
    }
}

impl Endec for u64 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let a = (*self >> 56) as u8;
        let b = ((*self >> 48) % 256) as u8;
        let c = ((*self >> 40) % 256) as u8;
        let d = ((*self >> 32) % 256) as u8;
        let e = ((*self >> 24) % 256) as u8;
        let f = ((*self >> 16) % 256) as u8;
        let g = ((*self >>  8) % 256) as u8;
        let h = (*self % 256) as u8;
        buffer.push(a);
        buffer.push(b);
        buffer.push(c);
        buffer.push(d);
        buffer.push(e);
        buffer.push(f);
        buffer.push(g);
        buffer.push(h);
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        if *index + 7 > buffer.len() {
            Err(EndecError::UnexpectedEof)
        } else {
            let a = buffer[*index];
            let b = buffer[*index + 1];
            let c = buffer[*index + 2];
            let d = buffer[*index + 3];
            let e = buffer[*index + 4];
            let f = buffer[*index + 5];
            let g = buffer[*index + 6];
            let h = buffer[*index + 7];
            *index += 8;

            Ok(
                ((a as u64) << 56)
                | ((b as u64) << 48)
                | ((c as u64) << 40)
                | ((d as u64) << 32)
                | ((e as u64) << 24)
                | ((f as u64) << 16)
                | ((g as u64) <<  8)
                | h as u64
            )
        }
    }
}

macro_rules! endec_for_signed {
    ($t1: ty, $t2: ty) => {
        impl Endec for $t1 {
            fn encode(&self, buffer: &mut Vec<u8>) {
                if *self < 0 {
                    if *self == <$t1>::MIN {
                        (<$t2>::MAX).encode(buffer);
                    } else {
                        (self.abs() as $t2 * 2 - 1).encode(buffer);
                    }
                } else {
                    (self.abs() as $t2 * 2).encode(buffer);
                }
            }
        
            fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
                let n = <$t2>::decode(buffer, index)?;
        
                if n % 2 == 0 {
                    Ok((n / 2) as $t1)
                } else {
                    Ok(-((n / 2) as $t1) - 1)
                }
            }
        }
    }
}

endec_for_signed!(i8, u8);
endec_for_signed!(i16, u16);
endec_for_signed!(i32, u32);
endec_for_signed!(i64, u64);
endec_for_signed!(isize, usize);

impl Endec for usize {
    fn encode(&self, buffer: &mut Vec<u8>) {
        (*self as u64).encode(buffer);
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        u64::decode(buffer, index).map(|n| n as usize)
    }
}
