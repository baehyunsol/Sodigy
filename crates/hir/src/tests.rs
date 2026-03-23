use crate::{
    Alias,
    Assert,
    Attribute,
    Block,
    Enum,
    Expr,
    Func,
    FuncShape,
    If,
    Let,
    Match,
    Pattern,
    Poly,
    Struct,
    StructShape,
    Type,
};
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<Alias>() < 160, "{}", size_of::<Alias>());
    assert!(size_of::<Assert>() < 160, "{}", size_of::<Assert>());
    assert!(size_of::<Attribute>() < 160, "{}", size_of::<Attribute>());
    assert!(size_of::<Block>() < 160, "{}", size_of::<Block>());
    assert!(size_of::<Enum>() < 160, "{}", size_of::<Enum>());
    assert!(size_of::<Expr>() < 160, "{}", size_of::<Expr>());
    assert!(size_of::<Func>() < 160, "{}", size_of::<Func>());
    assert!(size_of::<FuncShape>() < 160, "{}", size_of::<FuncShape>());
    assert!(size_of::<If>() < 160, "{}", size_of::<If>());
    assert!(size_of::<Let>() < 160, "{}", size_of::<Let>());
    assert!(size_of::<Match>() < 160, "{}", size_of::<Match>());
    assert!(size_of::<Pattern>() < 160, "{}", size_of::<Pattern>());
    assert!(size_of::<Poly>() < 160, "{}", size_of::<Poly>());
    assert!(size_of::<Struct>() < 160, "{}", size_of::<Struct>());
    assert!(size_of::<StructShape>() < 160, "{}", size_of::<StructShape>());
    assert!(size_of::<Type>() < 160, "{}", size_of::<Type>());
}
