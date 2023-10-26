mod fmt;

pub(crate) const STRING_B: u32 = 100;
pub(crate) const STRING_F: u32 = 101;
pub(crate) const DOTDOTDOT: u32 = 102;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct InternedString(pub(crate) u32);

impl From<u32> for InternedString {
    fn from(n: u32) -> Self {
        InternedString(n)
    }
}
