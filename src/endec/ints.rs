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

// TODO: use u64 instead of u32
impl Endec for usize {
    fn encode(&self, buffer: &mut Vec<u8>) {
        (*self as u32).encode(buffer);
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        u32::decode(buffer, index).map(|n| n as usize)
    }
}
