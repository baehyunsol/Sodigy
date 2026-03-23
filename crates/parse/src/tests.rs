use crate::{
    Alias,
    Assert,
    Attribute,
    Block,
    Enum,
    Expr,
    Field,
    Func,
    If,
    Lambda,
    Let,
    Match,
    Pattern,
    Struct,
    Type,
    Use,
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
    assert!(size_of::<Field>() < 160, "{}", size_of::<Field>());
    assert!(size_of::<Func>() < 160, "{}", size_of::<Func>());
    assert!(size_of::<If>() < 160, "{}", size_of::<If>());
    assert!(size_of::<Lambda>() < 160, "{}", size_of::<Lambda>());
    assert!(size_of::<Let>() < 160, "{}", size_of::<Let>());
    assert!(size_of::<Match>() < 160, "{}", size_of::<Match>());
    assert!(size_of::<Pattern>() < 160, "{}", size_of::<Pattern>());
    assert!(size_of::<Struct>() < 160, "{}", size_of::<Struct>());
    assert!(size_of::<Type>() < 160, "{}", size_of::<Type>());
    assert!(size_of::<Use>() < 160, "{}", size_of::<Use>());
}
