pub enum InternedString {
    // it must be a valid utf-8
    Short {
        length: u8,
        buffer: [u8; 8],
    },
    Long(u128),  // hash of the string
}

pub fn intern_string(s: &[u8]) -> Self {
    if s.len() < 9 {
        InternedString::Short {
            length: s.len() as u8,
            buffer: [
                *s.get(0).unwrap_or(&0),
                *s.get(1).unwrap_or(&0),
                *s.get(2).unwrap_or(&0),
                *s.get(3).unwrap_or(&0),
                *s.get(4).unwrap_or(&0),
                *s.get(5).unwrap_or(&0),
                *s.get(6).unwrap_or(&0),
                *s.get(7).unwrap_or(&0),
            ],
        }
    }

    else {
        InternedString::Long(hash(s))
    }
}

fn hash(s: &[u8]) -> u128 {
    let mut r = 0xffff_ffff_ffff_ffff_ffff;

    for (i, b) in s.iter().enumerate() {
        let mut k = b as u128;
        k |= ((i as u128) & 0xffff) << 16;
        k |= ((r >> 32) & 0xffff) << 32;
        k = 2 * k * k + k + 1;
        r += k;
    }

    r & 0xffff_ffff_ffff_ffff_ffff
}
