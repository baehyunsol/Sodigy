use super::{SdgHash, SdgHashResult};

impl SdgHash for &[u8] {
    fn sdg_hash(&self) -> SdgHashResult {
        let mut result = 0;

        for (i, c) in self.iter().enumerate() {
            result *= 279;
            result += *c as u128 + (i as u128) % 23;

            result %= 1 << 118;
        }

        SdgHashResult(result)
    }
}