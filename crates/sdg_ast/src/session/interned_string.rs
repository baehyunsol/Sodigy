use super::LocalParseSession;
use crate::utils::bytes_to_string;

/*
 * 0: dummy
 * 1 ~ 0xff_fff: builtins and preludes
 *    indices of builtins do not change across compilations, but it might change when the Sodigy version changes.
 *    InternedString is just name, it has nothing to do with its actual meaning.
 * 0x100_000 ~ 0x100_000 + keywords.len(): keywords
 * others: identifiers
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

    pub fn to_string(&self, session: &LocalParseSession) -> String {
        bytes_to_string(&session.unintern_string(*self))
    }
}

impl From<usize> for InternedString {
    fn from(n: usize) -> Self {
        InternedString(n as u32)
    }
}

impl From<InternedString> for usize {
    fn from(s: InternedString) -> usize {
        s.0 as usize
    }
}
