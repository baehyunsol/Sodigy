use crate::{
    Block,
    EnumShape,
    Expr,
    FuncShape,
    Pattern,
    Poly,
    StructShape,
    Type,
};
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<Block>() <= 256, "{}", size_of::<Block>());
    assert!(size_of::<EnumShape>() <= 256, "{}", size_of::<EnumShape>());

    // TODO: I really want it to be under 160, but 240 is the best I can get...
    assert!(size_of::<Expr>() <= 240, "{}", size_of::<Expr>());

    assert!(size_of::<FuncShape>() <= 256, "{}", size_of::<FuncShape>());

    // TODO: I can get this number if I change `kind: PatternKind` to `kind: Box<PatternKind>`.
    //       But then, the negative impact would be bigger...
    // assert!(size_of::<Pattern>() <= 160, "{}", size_of::<Pattern>());

    assert!(size_of::<Poly>() <= 160, "{}", size_of::<Poly>());
    assert!(size_of::<StructShape>() <= 256, "{}", size_of::<StructShape>());

    // TODO: I really want it to be under 160, but 240 is the best I can get...
    assert!(size_of::<Type>() <= 256, "{}", size_of::<Type>());
}
