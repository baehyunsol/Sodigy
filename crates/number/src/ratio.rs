use crate::big_int::BigInt;

// `denom` is always greater than or equal to 1.
// If `numer` is 0, `denom` must be 1.
#[derive(Clone, Debug)]
pub struct Ratio {
    pub numer: BigInt,
    pub denom: BigInt,
}
