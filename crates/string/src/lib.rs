// 0 A A A A A A A   ... (15 bytes actual data)
// 1 B B B B B B B   B B B B B B B B   B B B B B B B B   B B B B B B B B   ... (12 bytes hash value)

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct InternedString(pub u128);

impl InternedString {}

pub fn intern_string(s: &[u8]) -> InternedString {
    match s.len() {
        0..15 => intern_short_string(s),
        15..=2147483647 => {
            let hashed = hash(s);

            InternedString(u128::from_be_bytes([
                127 | (s.len() >> 24) as u8,
                ((s.len() >> 16) & 0xff) as u8,
                ((s.len() >> 8) & 0xff) as u8,
                (s.len() & 0xff) as u8,
                ((hashed >> 88) & 0xff) as u8,
                ((hashed >> 80) & 0xff) as u8,
                ((hashed >> 72) & 0xff) as u8,
                ((hashed >> 64) & 0xff) as u8,
                ((hashed >> 56) & 0xff) as u8,
                ((hashed >> 48) & 0xff) as u8,
                ((hashed >> 40) & 0xff) as u8,
                ((hashed >> 32) & 0xff) as u8,
                ((hashed >> 24) & 0xff) as u8,
                ((hashed >> 16) & 0xff) as u8,
                ((hashed >> 8) & 0xff) as u8,
                (hashed & 0xff) as u8,
            ]))
        },
        2147483648.. => todo!(),
    }
}

fn intern_short_string(s: &[u8]) -> InternedString {
    InternedString(u128::from_be_bytes([
        s.len() as u8,
        *s.get(0).unwrap_or(&0),
        *s.get(1).unwrap_or(&0),
        *s.get(2).unwrap_or(&0),
        *s.get(3).unwrap_or(&0),
        *s.get(4).unwrap_or(&0),
        *s.get(5).unwrap_or(&0),
        *s.get(6).unwrap_or(&0),
        *s.get(7).unwrap_or(&0),
        *s.get(8).unwrap_or(&0),
        *s.get(9).unwrap_or(&0),
        *s.get(10).unwrap_or(&0),
        *s.get(11).unwrap_or(&0),
        *s.get(12).unwrap_or(&0),
        *s.get(13).unwrap_or(&0),
        *s.get(14).unwrap_or(&0),
    ]))
}

fn hash(s: &[u8]) -> u128 {
    let mut r = 0;

    for (i, b) in s.iter().enumerate() {
        let c = (((r >> 24) & 0x00ff_ffff) << 24) | ((i & 0xfff) << 12) as u128 | *b as u128;
        let cc = c * c + c + 1;
        r += cc;
        r &= 0xffff_ffff_ffff_ffff_ffff_ffff;
    }

    r
}
