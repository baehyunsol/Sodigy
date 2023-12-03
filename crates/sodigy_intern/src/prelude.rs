use crate::{
    string::{DOTDOTDOT, EMPTY, UNDERBAR, STRING_B, STRING_F},
    InternedNumeric,
    InternedString,
    DATA_MASK,
    IS_INTEGER,
    IS_SMALL_INTEGER,
    unintern_numeric,
    unintern_string,
};
use sodigy_keyword::{Keyword, keywords};
use sodigy_test::sodigy_assert;

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
        sodigy_assert!(
            self.0 != STRING_B
            || unintern_string(*self) == b"b"
        );

        self.0 == STRING_B
    }

    // character 'f'
    pub fn is_f(&self) -> bool {
        sodigy_assert!(
            self.0 != STRING_F
            || unintern_string(*self) == b"f"
        );

        self.0 == STRING_F
    }

    pub fn is_underbar(&self) -> bool {
        sodigy_assert!(
            self.0 != UNDERBAR
            || unintern_string(*self) == b"_"
        );

        self.0 == UNDERBAR
    }

    pub fn is_empty(&self) -> bool {
        sodigy_assert!(
            self.0 != UNDERBAR
            || unintern_string(*self) == b""
        );

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

    pub fn is_zero(&self) -> bool {
        sodigy_assert!(
            self.0 != (0 | IS_INTEGER | IS_SMALL_INTEGER)
            || unintern_numeric(*self).is_zero()
        );

        self.0 == (0 | IS_INTEGER | IS_SMALL_INTEGER)
    }

    pub fn try_unwrap_small_int(&self) -> Option<u32> {
        if self.0 & IS_SMALL_INTEGER != 0 {
            Some(self.0 & DATA_MASK)
        }

        else {
            None
        }
    }

    pub fn try_unwrap_digits_and_exp_from_ratio(&self) -> Option<(Vec<u8>, i64)> {
        if self.is_integer() {
            None
        }

        else {
            let n = unintern_numeric(*self);

            Some(n.digits_and_exp())
        }
    }

    pub fn try_unwrap_digits_and_exp_from_int(&self) -> Option<(Vec<u8>, i64)> {
        if !self.is_integer() {
            None
        }

        else {
            let n = unintern_numeric(*self);

            Some(n.digits_and_exp())
        }
    }

    pub fn zero() -> Self {
        InternedNumeric(0 | IS_INTEGER | IS_SMALL_INTEGER)
    }
}
