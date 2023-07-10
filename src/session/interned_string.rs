#[cfg(test)]
use super::LocalParseSession;

/*
 * 0: dummy
 * 1 ~ 0xff_fff: builtins
 * indices of builtins do not change across compilations, but it might change when the compiler version changes
 * 0x100_000 ~ 0x100_000 + keywords.len(): keywords
 */
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct InternedString(u32);

const DUMMY_INDEX: u32 = 0;
pub const KEYWORD_START: u32 = 0x100_000;

impl InternedString {
    pub fn dummy() -> Self {
        InternedString(DUMMY_INDEX)
    }

    pub fn is_dummy(&self) -> bool {
        self.0 == DUMMY_INDEX
    }

    #[cfg(test)]
    pub fn to_string(&self, session: &LocalParseSession) -> String {
        String::from_utf8_lossy(&session.unintern_string(*self)).to_string()
    }
}

impl From<usize> for InternedString {
    fn from(n: usize) -> Self {
        assert!(n as u32 != DUMMY_INDEX, "Internal Compiler Error B2CC601");

        InternedString(n as u32)
    }
}

impl From<InternedString> for usize {
    fn from(s: InternedString) -> usize {
        s.0 as usize
    }
}
