use super::{Endec, EndecError};

impl<A: Endec> Endec for (A, ) {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.0.encode(buffer);
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        Ok((
            A::decode(buffer, index)?,
        ))
    }
}

impl<A: Endec, B: Endec> Endec for (A, B) {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.0.encode(buffer);
        self.1.encode(buffer);
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        Ok((
            A::decode(buffer, index)?,
            B::decode(buffer, index)?,
        ))
    }
}

impl<A: Endec, B: Endec, C: Endec> Endec for (A, B, C) {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.0.encode(buffer);
        self.1.encode(buffer);
        self.2.encode(buffer);
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        Ok((
            A::decode(buffer, index)?,
            B::decode(buffer, index)?,
            C::decode(buffer, index)?,
        ))
    }
}

impl<A: Endec, B: Endec, C: Endec, D: Endec> Endec for (A, B, C, D) {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.0.encode(buffer);
        self.1.encode(buffer);
        self.2.encode(buffer);
        self.3.encode(buffer);
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        Ok((
            A::decode(buffer, index)?,
            B::decode(buffer, index)?,
            C::decode(buffer, index)?,
            D::decode(buffer, index)?,
        ))
    }
}
