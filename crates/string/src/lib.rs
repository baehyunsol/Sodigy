// In Sodigy compiler, a string (whether it's an identifier or a literal) is
// always interned to a 16 bytes `InternedString`. If the string is short
// (less than 16 bytes), length (1 byte) and content is directly encoded to
// the interned_string. Otherwise, the content is stored in a file and its
// length and hash is encoded to the interned_string.

use sodigy_fs_api::{FileError, join};

mod endec;
mod fmt;
mod fs;

#[cfg(test)]
mod tests;

use fs::{insert_fs_map, read_fs_map};

/// - Type A: `0 0 0 0 L L L L   ...`
/// - Type B: `1 0 X X X X X X   L L L L L L L L   L L L L L L L L   L L L L L L L L   ...`
/// - Type C: `1 1 X X X X X X   L L L L L L L L   L L L L L L L L   L L L L L L L L   L L L L L L L L   L L L L L L L L   ...`
///
/// Type A is for short strings (less than 16 bytes). The first 4 bits are 0, next 4 bits
/// are the length of the string, and the remaining 15 bytes are the content.
/// Type B is for medium sized strings (16..=16777215 bytes). The first 2 bits are 0b10,
/// next 6 bits are the least significant bit of the first byte of the content (it's for
/// fast comparison), next 24 bits are the length of the string, and the remaining 12 bytes
/// are the hash of the content. The actual content is stored in a file, that's why functions
/// require `intermediate_dir` to intern/unintern strings.
/// Type C is for large strings (16777216..=1099511627775 bytes). The first 2 bits are 0b11,
/// next 6 bits are the least significant bit of the first byte of the content, next 40 bits
/// are the length of the content, and the remaining 10 bytes are for the hash of the content.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InternedString(pub u128);

impl InternedString {
    pub fn empty() -> Self {
        intern_short_string(b"")
    }

    pub fn try_unintern_short_string(&self) -> Option<Vec<u8>> {
        let length = match (self.0 >> 120) as u8 {
            x @ 0..=15 => x as usize,
            _ => {
                return None;
            },
        };
        let bytes = self.0.to_be_bytes();
        Some(bytes[1..(1 + length)].to_vec())
    }

    pub fn len(&self) -> usize {
        match self.0 >> 120 {
            l @ 0..=15 => l as usize,
            128..192 => ((self.0 >> 96) & 0x00ff_ffff) as usize,
            192.. => ((self.0 >> 80) & 0x00ff_ffff_ffff) as usize,

            // It's likely to be a dummy InternedString. It's a compiler bug
            // if you try to get the length of dummy.
            _ => unreachable!(),
        }
    }

    pub fn eq(&self, s: &[u8]) -> bool {
        match (self.len(), s.len()) {
            (l1, l2) if l1 != l2 => false,
            (l @ 0..=15, _) => &self.0.to_be_bytes()[1..(1 + l)] == s,
            (l, _) => {
                let preview = (self.0 >> 120) as u8 & 0x3f;

                if preview != s[0] & 0x3f {
                    false
                }

                else {
                    let hashed = hash(s);

                    match l {
                        ..16777216 => self.0 & 0xffff_ffff_ffff_ffff_ffff_ffff == hashed,
                        _ => self.0 & 0xffff_ffff_ffff_ffff_ffff == hashed & 0xffff_ffff_ffff_ffff_ffff,
                    }
                }
            },
        }
    }

    pub fn dummy() -> Self {
        // invalid InternedString
        InternedString(0x7fff_ffff_ffff_ffff_ffff_ffff_ffff_ffff)
    }

    pub fn is_dummy(&self) -> bool {
        self.0 == 0x7fff_ffff_ffff_ffff_ffff_ffff_ffff_ffff
    }

    /// It ignores all the errors.
    pub fn unintern_or_default(&self, intermediate_dir: &str) -> String {
        String::from_utf8_lossy(&unintern_string(*self, intermediate_dir).map(|s| s.unwrap_or(b"????".to_vec())).unwrap_or(b"????".to_vec())).to_string()
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
        0..=15 => Ok(intern_short_string(s)),
        16..=16777215 => {
            let hashed = hash(s);
            let id = InternedString(u128::from_be_bytes([
                128 | s[0] & 0x3f,
                (s.len() >> 16) as u8,
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
        // This pattern is supposed to be `16777216..=1099511627775`, but I'm
        // worried that the pattern would be a compile error on 32-bit systems...
        16777216.. => {
            let hashed = hash(s);
            let id = InternedString(u128::from_be_bytes([
                192 | s[0] & 0x3f,
                (s.len() >> 32) as u8,
                ((s.len() >> 24) & 0xff) as u8,
                ((s.len() >> 16) & 0xff) as u8,
                ((s.len() >> 8) & 0xff) as u8,
                (s.len() & 0xff) as u8,
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
