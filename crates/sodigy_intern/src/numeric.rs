mod fmt;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct InternedNumeric(pub(crate) u32);

impl From<u32> for InternedNumeric {
    fn from(n: u32) -> Self {
        InternedNumeric(n)
    }
}
