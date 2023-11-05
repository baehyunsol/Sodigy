use crate::{string::{DOTDOTDOT, EMPTY, UNDERBAR, STRING_B, STRING_F}, InternedNumeric, InternedString, IS_INTEGER};
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

    pub fn is_underbar(&self) -> bool {
        self.0 == UNDERBAR
    }

    pub fn is_empty(&self) -> bool {
        self.0 == EMPTY
    }

    pub fn dotdotdot() -> Self {
        InternedString(DOTDOTDOT)
    }
}

impl InternedNumeric {
    pub fn is_integer(&self) -> bool {
        self.0 & IS_INTEGER != 0
    }
}
