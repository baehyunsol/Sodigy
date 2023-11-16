use crate::{IS_INTEGER, PRELUDE_NUMERICS, unintern_numeric};
mod fmt;

pub(crate) const ZERO: u32 = 100 | IS_INTEGER | PRELUDE_NUMERICS;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct InternedNumeric(pub(crate) u32);

impl From<u32> for InternedNumeric {
    fn from(n: u32) -> Self {
        InternedNumeric(n)
    }
}

impl InternedNumeric {
    // quite slowish: it has to unintern numerics
    pub fn gt(&self, other: &Self) -> bool {
        if *self == *other {
            false
        }

        else {
            unintern_numeric(*self).gt(&unintern_numeric(*other))
        }
    }
}
