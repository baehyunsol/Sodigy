use crate::{
    Block,
    Expr,
    Intrinsic,
    Match,
    Type,
};
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<Block>() <= 256, "{}", size_of::<Block>());
    assert!(size_of::<Expr>() <= 240, "{}", size_of::<Expr>());
    assert!(size_of::<Intrinsic>() <= 8, "{}", size_of::<Intrinsic>());
    assert!(size_of::<Match>() <= 160, "{}", size_of::<Match>());
    assert!(size_of::<Type>() <= 160, "{}", size_of::<Type>());
}
