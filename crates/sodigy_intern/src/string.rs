use crate::SPECIAL_STRINGS;
mod fmt;

pub(crate) const STRING_B: u32 = 100 | SPECIAL_STRINGS;
pub(crate) const STRING_F: u32 = 101 | SPECIAL_STRINGS;
pub(crate) const DOTDOTDOT: u32 = 102 | SPECIAL_STRINGS;
pub(crate) const UNDERBAR: u32 = 103 | SPECIAL_STRINGS;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct InternedString(pub(crate) u32);

impl From<u32> for InternedString {
    fn from(n: u32) -> Self {
        InternedString(n)
    }
}
