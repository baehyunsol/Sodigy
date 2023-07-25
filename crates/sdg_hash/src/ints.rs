use super::{SdgHash, SdgHashResult};

// if m == n, hash(m as T) == hash(n as T) for all integer type T

const B_32: u32 = 1 << 24;
const B_64: u64 = 1 << 24;
const B_128: u128 = 1 << 24;
const BB_64: u64 = 1 << 48;
const BB_128: u128 = 1 << 48;
const BBB_128: u128 = 1 << 72;
const BBBB_128: u128 = 1 << 96;
const BBBBB_128: u128 = 1 << 120;

impl SdgHash for u8 {
    fn sdg_hash(&self) -> SdgHashResult {
        SdgHashResult(lfsr_24_3(*self as u32) as u128)
    }
}

impl SdgHash for u16 {
    fn sdg_hash(&self) -> SdgHashResult {
        SdgHashResult(lfsr_24_3(*self as u32) as u128)
    }
}

impl SdgHash for u32 {
    fn sdg_hash(&self) -> SdgHashResult {
        SdgHashResult(
            (
                (lfsr_24_3(*self / B_32) & 0xff) * B_32
                | lfsr_24_3(*self % B_32)
            ) as u128
        )
    }
}

impl SdgHash for u64 {
    fn sdg_hash(&self) -> SdgHashResult {
        SdgHashResult(
            (
                (lfsr_24_3((*self / BB_64) as u32) as u64 & 0xffff) * BB_64
                | lfsr_24_3((*self / B_64 % B_64) as u32) as u64 * B_64
                | lfsr_24_3((*self % B_64) as u32) as u64
            ) as u128
        )
    }
}

// TODO: what if the system is 128-bit?
impl SdgHash for usize {
    fn sdg_hash(&self) -> SdgHashResult {
        (*self as u64).sdg_hash()
    }
}

impl SdgHash for u128 {
    fn sdg_hash(&self) -> SdgHashResult {
        SdgHashResult(
            (lfsr_24_3((*self / BBBBB_128) as u32) as u128 & 0xff) * BBBBB_128
            | lfsr_24_3((*self / BBBB_128 % B_128) as u32) as u128 * BBBB_128
            | lfsr_24_3((*self / BBB_128 % B_128) as u32) as u128 * BBB_128
            | lfsr_24_3((*self / BB_128 % B_128) as u32) as u128 * BB_128
            | lfsr_24_3((*self / B_128 % B_128) as u32) as u128 * B_128
            | lfsr_24_3((*self % B_128) as u32) as u128
        )
    }
}

macro_rules! signed_sdg_hash {
    ($t1: ty, $t2: ty) => {
        impl SdgHash for $t1 {
            fn sdg_hash(&self) -> SdgHashResult {
                if *self < 0 {
                    (self.abs() as $t2).sdg_hash().sdg_hash()
                } else {
                    (*self as $t2).sdg_hash()
                }
            }
        }
    };
}

signed_sdg_hash!(i8, u32);
signed_sdg_hash!(i16, u32);
signed_sdg_hash!(i32, u32);
signed_sdg_hash!(i64, u64);
signed_sdg_hash!(isize, u64);
signed_sdg_hash!(i128, u128);

#[inline]
fn lfsr_24_3(n: u32) -> u32 {
    lfsr_24(lfsr_24(lfsr_24(n))) % B_32
}

fn lfsr_24(n: u32) -> u32 {
    let bit = ((n >> 0) ^ (n >> 1) ^ (n >> 2) ^ (n >> 7)) & 1;

    (n >> 1) | (bit << 23)
}

#[cfg(test)]
mod tests {
    use super::super::SdgHash;

    #[test]
    fn type_conversion() {
        for i in 0..65535 {
            assert!(
                (i as u16).sdg_hash() == (i as u32).sdg_hash()
                &&
                (i as u32).sdg_hash() == (i as u64).sdg_hash()
                &&
                (i as u64).sdg_hash() == (i as u128).sdg_hash()
            );
        }
    }
}