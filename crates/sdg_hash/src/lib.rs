mod bytes;
mod ints;

#[derive(Copy, Clone, PartialEq)]
pub struct SdgHashResult(u128);

impl SdgHashResult {
    pub fn to_bytes(&self) -> Vec<u8> {
        vec![
            (self.0 >> 124) as u8 + b'a',
            to_char(((self.0 >> 119) % 32) as u8),
            to_char(((self.0 >> 114) % 32) as u8),
            to_char(((self.0 >> 109) % 32) as u8),

            to_char(((self.0 >> 104) % 32) as u8),
            to_char(((self.0 >> 99) % 32) as u8),
            to_char(((self.0 >> 94) % 32) as u8),
            to_char(((self.0 >> 89) % 32) as u8),
            to_char(((self.0 >> 84) % 32) as u8),
            to_char(((self.0 >> 79) % 32) as u8),
            to_char(((self.0 >> 74) % 32) as u8),

            to_char(((self.0 >> 69) % 32) as u8),
            to_char(((self.0 >> 64) % 32) as u8),
            to_char(((self.0 >> 59) % 32) as u8),
            to_char(((self.0 >> 54) % 32) as u8),
            to_char(((self.0 >> 49) % 32) as u8),
            to_char(((self.0 >> 44) % 32) as u8),
            to_char(((self.0 >> 39) % 32) as u8),
            to_char(((self.0 >> 34) % 32) as u8),
            to_char(((self.0 >> 29) % 32) as u8),

            to_char(((self.0 >> 24) % 32) as u8),
            to_char(((self.0 >> 19) % 32) as u8),
            to_char(((self.0 >> 14) % 32) as u8),
            to_char(((self.0 >>  9) % 32) as u8),
            to_char(((self.0 >>  4) % 32) as u8),
            to_char((self.0 % 16) as u8),
        ]
    }

    pub fn to_string(&self) -> String {
        String::from_utf8(self.to_bytes()).expect("Internal Compiler Error 0CF2EF4")
    }
}

impl std::ops::BitXor<SdgHashResult> for SdgHashResult {
    type Output = SdgHashResult;

    fn bitxor(self, rhs: SdgHashResult) -> Self::Output {
        SdgHashResult(self.0 ^ rhs.0)
    }
}

impl SdgHash for SdgHashResult {
    fn sdg_hash(&self) -> SdgHashResult {
        self.0.sdg_hash()
    }
}

pub trait SdgHash {
    fn sdg_hash(&self) -> SdgHashResult;
}

#[inline]
fn to_char(n: u8) -> u8 {

    if n < 10 {
        n + b'0'
    } else {
        n - 10 + b'a'
    }

}