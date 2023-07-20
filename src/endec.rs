// TODO: independent crate

mod collections;
mod err;
mod ints;
mod string;
mod tuples;

pub use err::EndecError;

pub trait Endec {
    fn encode(&self, buffer: &mut Vec<u8>);
    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> where Self: Sized;
}

impl Endec for bool {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.push(*self as u8);
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            Some(n) => Err(EndecError::UnexpectedByte(*n)),
            None => Err(EndecError::UnexpectedEof),
        }
    }
}

impl<A: Endec> Endec for &A {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.encode(buffer);
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        Self::decode(buffer, index)
    }
}
