use super::{Endec, EndecError};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

impl<A: Endec> Endec for Vec<A> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.len().encode(buffer);

        for elem in self.iter() {
            elem.encode(buffer);
        }
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        let len = usize::decode(buffer, index)?;
        let mut result = Vec::with_capacity(len);

        for _ in 0..len {
            result.push(A::decode(buffer, index)?);
        }

        Ok(result)
    }
}

impl<A: Endec + Hash + Eq> Endec for HashSet<A> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.len().encode(buffer);

        for elem in self.iter() {
            elem.encode(buffer);
        }
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        let len = usize::decode(buffer, index)?;
        let mut result = HashSet::with_capacity(len);

        for _ in 0..len {
            result.insert(A::decode(buffer, index)?);
        }

        Ok(result)
    }
}

impl<K: Endec + Hash + Eq, V: Endec + Hash + Eq> Endec for HashMap<K, V> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.len().encode(buffer);

        for elem in self.iter() {
            elem.encode(buffer);
        }
    }

    fn decode(buffer: &[u8], index: &mut usize) -> Result<Self, EndecError> {
        let len = usize::decode(buffer, index)?;
        let mut result = HashMap::with_capacity(len);

        for _ in 0..len {
            let (key, value) = (K::decode(buffer, index)?, V::decode(buffer, index)?);
            result.insert(key ,value);
        }

        Ok(result)
    }
}
