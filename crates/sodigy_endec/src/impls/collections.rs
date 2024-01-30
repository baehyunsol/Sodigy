use crate::{Endec, EndecError, EndecSession};
use smallvec::SmallVec;
use std::collections::{HashMap, HashSet};

impl <T: Endec + std::hash::Hash + std::cmp::Eq, U: Endec> Endec for HashMap<T, U> {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.len().encode(buf, session);

        for (k, v) in self.iter() {
            k.encode(buf, session);
            v.encode(buf, session);
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        let len = usize::decode(buf, index, session)?;
        let mut result = HashMap::with_capacity(len);

        for _ in 0..len {
            let k = T::decode(buf, index, session)?;
            let v = U::decode(buf, index, session)?;
            result.insert(k, v);
        }

        Ok(result)
    }
}

impl <T: Endec + std::hash::Hash + std::cmp::Eq> Endec for HashSet<T> {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.len().encode(buf, session);

        for e in self.iter() {
            e.encode(buf, session);
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        let len = usize::decode(buf, index, session)?;
        let mut result = HashSet::with_capacity(len);

        for _ in 0..len {
            result.insert(T::decode(buf, index, session)?);
        }

        Ok(result)
    }
}

macro_rules! vec_like {
    (small_vec, $n: expr) => {
        vec_like!(SmallVec, SmallVec<[T; $n]>, [T; $n]);
    };
    ($name: ident, $t: ty, $tt: ty) => {
        impl <T: Endec> Endec for $t {
            fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
                self.len().encode(buf, session);
        
                for v in self.iter() {
                    v.encode(buf, session);
                }
            }
        
            fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
                let len = usize::decode(buf, index, session)?;
                let mut result = $name::<$tt>::with_capacity(len);
        
                for _ in 0..len {
                    result.push(T::decode(buf, index, session)?);
                }
        
                Ok(result)
            }
        }
    };
}

vec_like!(Vec, Vec<T>, T);
vec_like!(small_vec,  0); vec_like!(small_vec,  1); vec_like!(small_vec,  2); vec_like!(small_vec,  3);
vec_like!(small_vec,  4); vec_like!(small_vec,  5); vec_like!(small_vec,  6); vec_like!(small_vec,  7);
vec_like!(small_vec,  8); vec_like!(small_vec,  9); vec_like!(small_vec, 10); vec_like!(small_vec, 11);
vec_like!(small_vec, 12); vec_like!(small_vec, 13); vec_like!(small_vec, 14); vec_like!(small_vec, 15);
vec_like!(small_vec, 16); vec_like!(small_vec, 17); vec_like!(small_vec, 18); vec_like!(small_vec, 19);
vec_like!(small_vec, 20); vec_like!(small_vec, 21); vec_like!(small_vec, 22); vec_like!(small_vec, 23);
