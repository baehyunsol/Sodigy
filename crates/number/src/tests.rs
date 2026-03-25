use crate::{
    InternedNumber,
    // BigInt,
    // Ratio,
};
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<InternedNumber>() <= 16, "{}", size_of::<InternedNumber>());

    // It's okay for these to be big! That's why `InternedNumber` exists...
    // assert!(size_of::<BigInt>() <= 48, "{}", size_of::<BigInt>());
    // assert!(size_of::<Ratio>() <= 48, "{}", size_of::<Ratio>());
}

#[test]
fn interned_small_integer() {
    let n = InternedNumber::from_u32(30, true);
    assert_eq!(i32::try_from(n).unwrap() as u32, 30);
    assert_eq!(i64::try_from(n).unwrap() as u32, 30);
    assert_eq!(u32::try_from(n).unwrap(), 30);
    assert_eq!(u64::try_from(n).unwrap() as u32, 30);

    let n = InternedNumber::from_u32(0, true);
    assert_eq!(i32::try_from(n).unwrap() as u32, 0);
    assert_eq!(i64::try_from(n).unwrap() as u32, 0);
    assert_eq!(u32::try_from(n).unwrap(), 0);
    assert_eq!(u64::try_from(n).unwrap() as u32, 0);

    let n = InternedNumber::from_i32(-30, true);
    assert_eq!(i32::try_from(n).unwrap(), -30);
    assert_eq!(i64::try_from(n).unwrap() as i32, -30);

    let n = InternedNumber::from_i32(0, true);
    assert_eq!(i32::try_from(n).unwrap(), 0);
    assert_eq!(i64::try_from(n).unwrap() as i32, 0);
    assert_eq!(u32::try_from(n).unwrap() as i32, 0);
    assert_eq!(u64::try_from(n).unwrap() as i32, 0);
}
