use crate::Error;
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<Error>() < 256, "{}", size_of::<Error>());
}
