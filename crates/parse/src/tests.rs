use crate::{
    Expr,
    Field,
    Path,
    Pattern,
    Type,
};
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<Expr>() <= 160, "{}", size_of::<Expr>());
    assert!(size_of::<Field>() <= 96, "{}", size_of::<Field>());
    assert!(size_of::<Path>() <= 96, "{}", size_of::<Path>());

    // TODO: I want it to be less than or equal to 160, but I can't reduce it anymore...
    assert!(size_of::<Pattern>() <= 256, "{}", size_of::<Pattern>());

    assert!(size_of::<Type>() <= 192, "{}", size_of::<Type>());
}
