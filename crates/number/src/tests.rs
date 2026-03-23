use crate::{
    BigInt,
    InternedNumber,
    Ratio,
};
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<BigInt>() < 48, "{}", size_of::<BigInt>());
    assert!(size_of::<InternedNumber>() < 24, "{}", size_of::<InternedNumber>());
    assert!(size_of::<Ratio>() < 48, "{}", size_of::<Ratio>());
}
