use crate::{intern_numeric, unintern_numeric};
use crate::prelude::{DATA_BIT_WIDTH, IS_INTEGER, IS_SMALL_INTEGER};

mod fmt;

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

    // quite slowish: it has to unintern numerics
    pub fn neg(&self) -> Self {
        intern_numeric(unintern_numeric(*self).neg())
    }

    // quite slowish: it has to unintern and intern numerics
    pub fn get_denom_and_numer(&self) -> (InternedNumeric, InternedNumeric) {
        if let Some(n) = self.try_unwrap_small_integer() {
            (
                InternedNumeric(1 | IS_INTEGER | IS_SMALL_INTEGER),
                InternedNumeric(n | IS_INTEGER | IS_SMALL_INTEGER),
            )
        }

        else {
            let n = unintern_numeric(*self);
            let (denom, numer) = n.get_denom_and_numer();

            debug_assert!(denom.is_integer());
            debug_assert!(numer.is_integer());
            debug_assert!(intern_numeric(denom.clone()).is_integer());
            debug_assert!(intern_numeric(numer.clone()).is_integer());

            (
                intern_numeric(denom),
                intern_numeric(numer),
            )
        }
    }
}

pub fn try_intern_small_integer(n: u32) -> Option<InternedNumeric> {
    if n < (1 << DATA_BIT_WIDTH) {
        Some(InternedNumeric(n | IS_INTEGER | IS_SMALL_INTEGER))
    }

    else {
        None
    }
}
