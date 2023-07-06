#[cfg(test)]
use super::LocalParseSession;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct InternedString(u32);

const DUMMY_INDEX: u32 = u32::MAX - 8;

impl InternedString {
    pub fn dummy() -> Self {
        InternedString(DUMMY_INDEX)
    }

    pub fn is_dummy(&self) -> bool {
        self.0 == DUMMY_INDEX
    }

    #[cfg(test)]
    pub fn to_string(&self, session: &LocalParseSession) -> String {
        let bytes = session.unintern_string(*self)
            .expect("Internal Compiler Error 657FECD");

        String::from_utf8_lossy(&bytes).to_string()
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
