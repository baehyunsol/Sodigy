use crate::prelude::{
    DATA_BIT_WIDTH,
    FOUR_BYTES_CHAR,
    IS_SHORT_STRING,
    STARTS_WITH_AT,
};

mod fmt;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct InternedString(pub(crate) u32);

impl From<u32> for InternedString {
    fn from(n: u32) -> Self {
        InternedString(n)
    }
}

pub fn try_intern_short_string(s: &[u8]) -> Option<InternedString> {
    if s.len() < 4 {
        let mut res = IS_SHORT_STRING | ((s.len() as u32) << (DATA_BIT_WIDTH + 2));

        let mut step = 16;
        let mut index = 0;

        while s.len() > index {
            res |= (s[index] as u32) << step;

            index += 1;
            step -= 8;
        }

        if s.starts_with(b"@") {
            res |= STARTS_WITH_AT;
        }

        Some(InternedString(res))
    }

    // a single character but it takes 4 bytes
    else if s.len() == 4 && s[0] >= 240 {
        Some(InternedString(
            FOUR_BYTES_CHAR | ((s[0] as u32 & 0x7) << 18)
            | ((s[1] as u32 & 0x3f) << 12)
            | ((s[2] as u32 & 0x3f) << 6)
            | (s[3] as u32 & 0x3f)
        ))
    }

    else {
        None
    }
}
