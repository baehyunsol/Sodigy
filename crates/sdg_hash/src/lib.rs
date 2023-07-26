mod bytes;
mod ints;
mod sha256;

#[cfg(test)]
mod tests;

pub(crate) use sha256::Sha256;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SdgHashResult([u8; 32]);

impl SdgHashResult {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.iter().map(
            |b| {
                let (a, b) =  (*b / 16, *b % 16);

                [to_char(a), to_char(b)]
            }
        ).collect::<Vec<[u8; 2]>>().concat()
    }

    pub fn to_string(&self) -> String {
        String::from_utf8(self.to_bytes()).expect("Internal Compiler Error 732856577C3")
    }

    pub fn to_u128(&self) -> u128 {
        u128::from_ne_bytes([
            self.0[0], self.0[1],
            self.0[2], self.0[3],
            self.0[4], self.0[5],
            self.0[6], self.0[7],
            self.0[8], self.0[9],
            self.0[10], self.0[11],
            self.0[12], self.0[13],
            self.0[14], self.0[15],
        ])
    }

    pub fn to_u64(&self) -> u64 {
        u64::from_ne_bytes([
            self.0[0], self.0[1],
            self.0[2], self.0[3],
            self.0[4], self.0[5],
            self.0[6], self.0[7],
        ])
    }
}

impl std::ops::BitXor<SdgHashResult> for SdgHashResult {
    type Output = SdgHashResult;

    fn bitxor(self, rhs: SdgHashResult) -> Self::Output {
        (self.to_u128() ^ rhs.to_u128()).sdg_hash()
    }
}

impl SdgHash for SdgHashResult {
    fn sdg_hash(&self) -> SdgHashResult {
        (&self.0 as &[u8]).sdg_hash()
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