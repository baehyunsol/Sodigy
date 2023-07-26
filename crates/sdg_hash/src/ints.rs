use super::{SdgHash, SdgHashResult, Sha256};

// if m == n, hash(m as T) == hash(n as T) for all integer type T

macro_rules! sdg_hash_uint {
    ($ty: ty) => {
        impl SdgHash for $ty {
            fn sdg_hash(&self) -> SdgHashResult {
                let mut n = *self;
                let mut s = Sha256::default();

                while n > 65535 {
                    let a = (n & 0xff00) >> 8;
                    let b = n & 0x00ff;

                    // n >> 16 doesn't work with u16
                    n >>= 8;
                    n >>= 8;

                    s.update(&[a as u8, b as u8]);
                }

                let a = (n & 0xff00) >> 8;
                let b = n & 0x00ff;
                s.update(&[a as u8, b as u8]);

                SdgHashResult(s.finish())
            }
        }
    }
}

sdg_hash_uint!(u16);
sdg_hash_uint!(u32);
sdg_hash_uint!(u64);
sdg_hash_uint!(usize);
sdg_hash_uint!(u128);
