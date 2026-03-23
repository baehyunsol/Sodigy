use crate::{
    Assert,
    Block,
    Enum,
    Expr,
    Func,
    If,
    Intrinsic,
    Let,
    Match,
    Pattern,
    Struct,
    Type,
};
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<Assert>() < 160, "{}", size_of::<Assert>());
    assert!(size_of::<Block>() < 160, "{}", size_of::<Block>());
    assert!(size_of::<Enum>() < 160, "{}", size_of::<Enum>());
    assert!(size_of::<Expr>() < 160, "{}", size_of::<Expr>());
    assert!(size_of::<Func>() < 160, "{}", size_of::<Func>());
    assert!(size_of::<If>() < 160, "{}", size_of::<If>());
    assert!(size_of::<Intrinsic>() < 160, "{}", size_of::<Intrinsic>());
    assert!(size_of::<Let>() < 160, "{}", size_of::<Let>());
    assert!(size_of::<Match>() < 160, "{}", size_of::<Match>());
    assert!(size_of::<Pattern>() < 160, "{}", size_of::<Pattern>());
    assert!(size_of::<Struct>() < 160, "{}", size_of::<Struct>());
    assert!(size_of::<Type>() < 160, "{}", size_of::<Type>());
}
