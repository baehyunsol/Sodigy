use super::{ENUM, UID, ZERO};

// UIDs of preludes and builtins

// type of every type
// e.g. `Option.is_subtype_of(Type) == Bool.True`
pub fn type_() -> UID {
    UID((0x0001 << 32) & ZERO | ENUM)
}
