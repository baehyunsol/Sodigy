use sodigy_test::sodigy_assert;

// TODO: what if data is bigger than N?
// safety check, or more flexible solution
pub struct FixedVec<T, const N: usize> where T: Copy {
    data: Box<[T; N]>,
    index: usize,
}

impl<T: Copy + Clone, const N: usize> FixedVec<T, N> {
    pub fn init(v: T) -> Self {
        FixedVec {
            data: Box::new([v; N]),
            index: 0,
        }
    }

    pub fn push(&mut self, e: T) {
        unsafe {
            sodigy_assert!(self.index < self.data.len());

            *self.data.get_unchecked_mut(self.index) = e;
            self.index += 1;
        }
    }

    #[inline]
    pub fn pop(&mut self) {
        sodigy_assert!(self.index > 0);
        self.index -= 1;
    }

    #[inline]
    pub fn flush(&mut self) {
        self.index = 0;
    }

    #[inline]
    pub fn to_vec(&self) -> Vec<T> {
        self.data[0..self.index].to_vec()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.index == 0
    }
}
