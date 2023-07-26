use super::{SdgHash, SdgHashResult, Sha256};

impl SdgHash for &[u8] {
    fn sdg_hash(&self) -> SdgHashResult {
        let mut s = Sha256::default();
        s.update(self);
        SdgHashResult(s.finish())
    }
}

impl SdgHash for &str {
    fn sdg_hash(&self) -> SdgHashResult {
        self.as_bytes().sdg_hash()
    }
}
