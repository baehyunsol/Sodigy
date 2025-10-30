use crate::{DecodeError, Endec};
use std::collections::HashMap;
use std::hash::Hash;

impl<T: Endec> Endec for Vec<T> {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.len().encode_impl(buffer);

        for e in self.iter() {
            e.encode_impl(buffer);
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (len, mut cursor) = usize::decode_impl(buffer, cursor)?;
        let mut result = Vec::with_capacity(len);

        for _ in 0..len {
            let (e, cursor_) = T::decode_impl(buffer, cursor)?;
            cursor = cursor_;
            result.push(e);
        }

        Ok((result, cursor))
    }
}

impl<K: Endec + Eq + Hash, V: Endec> Endec for HashMap<K, V> {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.len().encode_impl(buffer);

        for (k, v) in self.iter() {
            k.encode_impl(buffer);
            v.encode_impl(buffer);
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (len, mut cursor) = usize::decode_impl(buffer, cursor)?;
        let mut result = HashMap::with_capacity(len);

        for _ in 0..len {
            let (k, cursor_) = K::decode_impl(buffer, cursor)?;
            cursor = cursor_;
            let (v, cursor_) = V::decode_impl(buffer, cursor)?;
            cursor = cursor_;

            result.insert(k, v);
        }

        Ok((result, cursor))
    }
}
