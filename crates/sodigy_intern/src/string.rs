use crate::PRELUDE_STRINGS;
mod fmt;

pub(crate) const EMPTY: u32 = 100 | PRELUDE_STRINGS;
pub(crate) const STRING_B: u32 = 101 | PRELUDE_STRINGS;
pub(crate) const STRING_F: u32 = 102 | PRELUDE_STRINGS;
pub(crate) const DOTDOTDOT: u32 = 103 | PRELUDE_STRINGS;
pub(crate) const UNDERBAR: u32 = 104 | PRELUDE_STRINGS;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct InternedString(pub(crate) u32);

impl From<u32> for InternedString {
    fn from(n: u32) -> Self {
        InternedString(n)
    }
}
