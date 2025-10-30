// In Sodigy compiler, a string (whether it's an identifier or a literal) is
// always interned to a 16 bytes `InternedString`. If the string is short
// (less than 16 bytes), length (1 byte) and content is directly encoded to
// the interned_string. Otherwise, the content is stored in a file and its
// length and hash is encoded to the interned_string.

use sodigy_fs_api::{FileError, join};

mod endec;
mod fmt;
mod fs;

use fs::{insert_fs_map, read_fs_map};

// 0 A A A A A A A   ... (15 bytes actual data)
// 1 B B B B B B B   B B B B B B B B   B B B B B B B B   B B B B B B B B   ... (12 bytes hash value)
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InternedString(pub u128);

impl InternedString {
    pub fn empty() -> Self {
        intern_short_string(b"")
    }

    pub fn is_short_string(&self) -> bool {
        self.0 < (1 << 127)
    }

    pub fn try_unintern_short_string(&self) -> Option<Vec<u8>> {
        let length = match (self.0 >> 120) as u8 {
            x @ (0..=15) => x as usize,
            _ => {
                return None;
            },
        };
        let bytes = self.0.to_be_bytes();
        Some(bytes[1..(1 + length)].to_vec())
    }

    pub fn length(&self) -> usize {
        if self.is_short_string() {
            (self.0 >> 120) as usize
        }

        else {
            (self.0 >> 96) as usize & 0x7fff_ffff
        }
    }

    pub fn dummy() -> Self {
        // invalid InternedString
        InternedString(0x7fff_ffff_ffff_ffff_ffff_ffff_ffff_ffff)
    }

    pub fn is_dummy(&self) -> bool {
        self.0 == 0x7fff_ffff_ffff_ffff_ffff_ffff_ffff_ffff
    }
}

pub fn unintern_string(s: InternedString, intermediate_dir: &str) -> Result<Option<Vec<u8>>, FileError> {
    match s.try_unintern_short_string() {
        Some(s) => Ok(Some(s)),
        None => {
            let map_dir = join(intermediate_dir, "str")?;
            read_fs_map(&map_dir, s)
        },
    }
}

pub fn intern_string(s: &[u8], intermediate_dir: &str) -> Result<InternedString, FileError> {
    match s.len() {
        0..15 => Ok(intern_short_string(s)),
        15..=2147483647 => {
            let hashed = hash(s);
            let id = InternedString(u128::from_be_bytes([
                128 | (s.len() >> 24) as u8,
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
            ]));
            let map_dir = join(intermediate_dir, "str")?;
            insert_fs_map(&map_dir, id, s)?;

            Ok(id)
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
