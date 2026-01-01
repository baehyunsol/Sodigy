use super::InternedString;
use std::fmt;

impl fmt::Debug for InternedString {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match (self.0 >> 120) as u8 {
            0..=15 => format!(
                "ShortString(b{:?})",
                String::from_utf8_lossy(&self.try_unintern_short_string().unwrap()),
            ),
            127 => String::from("DummyString()"),
            128.. => {
                let (len, hash) = match self.len() {
                    0..=15 => unreachable!(),
                    len @ 16..16777216 => (
                        len,
                        format!("{:024x}", self.0 & 0xffff_ffff_ffff_ffff_ffff_ffff),
                    ),
                    len @ 16777216.. => (
                        len,
                        format!("...{:020x}", self.0 & 0xffff_ffff_ffff_ffff_ffff),
                    ),
                };

                format!("LongString {{ length: {len}, hash: {hash} }}")
            },
            _ => unreachable!(),
        };

        write!(fmt, "{s}")
    }
}
