use crate::{
    IdentWithOrigin,
    NameKind,
    NameOrigin,
    Namespace,
};
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<IdentWithOrigin>() <= 128, "{}", size_of::<IdentWithOrigin>());
    assert!(size_of::<NameKind>() <= 64, "{}", size_of::<NameKind>());
    assert!(size_of::<NameOrigin>() <= 64, "{}", size_of::<NameOrigin>());
    assert!(size_of::<Namespace>() <= 128, "{}", size_of::<Namespace>());
}
