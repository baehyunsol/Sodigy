use crate::FixedVec;

pub trait VecLike {
    type Element;

    fn vl_push(&mut self, e: Self::Element);
    fn vl_len(&self) -> usize;

    fn vl_is_empty(&self) -> bool {
        self.vl_len() == 0
    }
}

impl<T: Copy, const N: usize> VecLike for FixedVec<T, N> {
    type Element = T;

    fn vl_push(&mut self, e: T) {
        self.push(e);
    }

    fn vl_len(&self) -> usize {
        self.len()
    }
}

impl<T> VecLike for Vec<T> {
    type Element = T;

    fn vl_push(&mut self, e: T) {
        self.push(e);
    }

    fn vl_len(&self) -> usize {
        self.len()
    }
}
