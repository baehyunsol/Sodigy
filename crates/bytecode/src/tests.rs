use crate::{Bytecode, Memory, Offset, Value};
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<Bytecode>() <= 160, "{}", size_of::<Bytecode>());
    assert!(size_of::<Memory>() <= 160, "{}", size_of::<Memory>());
    assert!(size_of::<Offset>() <= 160, "{}", size_of::<Offset>());
    assert!(size_of::<Value>() <= 160, "{}", size_of::<Value>());
}
