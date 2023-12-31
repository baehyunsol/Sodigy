use crate::{
    InternedNumeric,
    InternedString,
    unintern_numeric,
};
use sodigy_keyword::{Keyword, keywords};
use sodigy_test::sodigy_assert;

const KEYWORD_LEN: usize = keywords().len();

pub(crate) const DATA_BIT_WIDTH: u32 = 26;
pub(crate) const DATA_MASK: u32 = !(0b111_111 << DATA_BIT_WIDTH);

// if the representation of the string in utf-8 is shorter than 4 bytes,
// the string is encoded directly into the data
// keywords are not encoded this way
pub(crate) const IS_SHORT_STRING: u32 = 0b100_000 << DATA_BIT_WIDTH;
pub(crate) const SHORT_STRING_LENGTH_MASK: u32 = 0b001_100 << DATA_BIT_WIDTH;

// 🦫
pub(crate) const FOUR_BYTES_CHAR: u32 = 0b010_000 << DATA_BIT_WIDTH;

// metadata for numerics
pub(crate) const IS_INTEGER: u32 = 0b100_000 << DATA_BIT_WIDTH;

// small enough to encode in 26 bits
pub(crate) const IS_SMALL_INTEGER: u32 = 0b010_000 << DATA_BIT_WIDTH;

impl InternedString {
    pub fn try_into_keyword(&self) -> Option<Keyword> {
        if self.is_normal_string() && (self.0 & DATA_MASK) < KEYWORD_LEN as u32 {
            Some(keywords()[self.0 as usize])
        }

        else {
            None
        }
    }

    pub fn is_short_string(&self) -> bool {
        self.0 & IS_SHORT_STRING != 0
    }

    pub fn is_4_bytes_char(&self) -> bool {
        self.0 & FOUR_BYTES_CHAR != 0
    }

    pub fn is_normal_string(&self) -> bool {
        (self.0 >> 30) == 0
    }

    #[inline]
    pub fn get_short_string_length(&self) -> usize {
        ((self.0 & SHORT_STRING_LENGTH_MASK) >> (DATA_BIT_WIDTH + 2)) as usize
    }

    pub fn try_unwrap_short_string(&self) -> Option<(/* length */ usize, /* data */ [u8; 4])> {
        if self.is_short_string() {
            let length = self.get_short_string_length();
            let data = [
                ((self.0 & 0xff0000) >> 16) as u8,
                ((self.0 & 0xff00) >> 8) as u8,
                (self.0 & 0xff) as u8,
                0,
            ];
            Some((length, data))
        }

        else if self.is_4_bytes_char() {
            Some((
                4,
                [
                    0b11110000 | ((self.0 >> 18) & 0x7) as u8,
                    0b10000000 | ((self.0 >> 12) & 0x3f) as u8,
                    0b10000000 | ((self.0 >> 6) & 0x3f) as u8,
                    0b10000000 | (self.0 & 0x3f) as u8,
                ],
            ))
        }

        else {
            None
        }
    }

    pub fn is_underbar(&self) -> bool {
        self.0 == IS_SHORT_STRING | (1 << (DATA_BIT_WIDTH + 2)) | ((b'_' as u32) << 16)
    }

    pub fn is_empty(&self) -> bool {
                         // short_string  four_bytes_char  length
        (self.0 >> 28) == 0b1_____________0________________00
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

    pub fn try_unwrap_small_integer(&self) -> Option<u32> {
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
