use crate::{string::{DOTDOTDOT, STRING_B, STRING_F}, InternedString};
use sodigy_keyword::{Keyword, keywords};

const KEYWORD_LEN: usize = keywords().len();

impl InternedString {
    pub fn try_into_keyword(&self) -> Option<Keyword> {
        if self.0 < KEYWORD_LEN as u32 {
            Some(keywords()[self.0 as usize])
        }

        else {
            None
        }
    }

    // character 'b'
    pub fn is_b(&self) -> bool {
        self.0 == STRING_B
    }

    // character 'f'
    pub fn is_f(&self) -> bool {
        self.0 == STRING_F
    }

    pub fn dotdotdot() -> Self {
        InternedString(DOTDOTDOT)
    }
}