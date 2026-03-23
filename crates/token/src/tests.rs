use crate::{
    Constant,
    Delim,
    InfixOp,
    Keyword,
    PostfixOp,
    PrefixOp,
    Punct,
};
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<Constant>() < 64, "{}", size_of::<Constant>());
    assert!(size_of::<Delim>() < 32, "{}", size_of::<Delim>());
    assert!(size_of::<InfixOp>() < 32, "{}", size_of::<InfixOp>());
    assert!(size_of::<Keyword>() < 32, "{}", size_of::<Keyword>());
    assert!(size_of::<PostfixOp>() < 32, "{}", size_of::<PostfixOp>());
    assert!(size_of::<PrefixOp>() < 32, "{}", size_of::<PrefixOp>());
    assert!(size_of::<Punct>() < 32, "{}", size_of::<Punct>());
}
