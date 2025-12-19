use crate::big_int::BigInt;

pub mod op;

// `denom` is always greater than or equal to 0.
// If `numer` is 0, `denom` must be 1.
// If `denom` is 0, `numer` is either 1 (pos-inf) or -1 (neg-inf).
#[derive(Clone, Debug)]
pub struct Ratio {
    pub numer: BigInt,
    pub denom: BigInt,
}
