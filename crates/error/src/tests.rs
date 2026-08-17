use crate::{Error, FuncEffect};
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<Error>() <= 256, "{}", size_of::<Error>());
    assert!(size_of::<FuncEffect>() <= 16, "{}", size_of::<FuncEffect>());
}
