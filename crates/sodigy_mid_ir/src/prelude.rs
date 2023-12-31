use crate::ty::Type;
use sodigy_uid::Uid;

// Uid of prelude objects are in this module
pub mod uids;

pub struct PreludeData {
    pub uid: Uid,
    pub ty: Type,
}
