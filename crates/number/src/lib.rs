mod base;

pub use base::Base;

pub enum InternedNumber {
    SmallInteger(i64),
    SmallRatio {
        denom: u32,
        numer: i32,
    },
}

pub fn intern_number(base: Base, integer: &[u8], frac: &[u8]) -> InternedNumber {
    todo!()
}
