use super::InternedString;
use std::fmt;

impl fmt::Debug for InternedString {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match (self.0 >> 120) as u8 {
            0..=127 => format!(
                "ShortString(b{:?})",
                String::from_utf8_lossy(&self.try_unintern_short_string().unwrap()),
            ),
            128.. => {
                let length = (self.0 >> 96) & 0x7fff_ffff;
                format!(
                    "LongString {} length: {length}, hash: {}... {}",
                    "{",
                    format!(
                        "{:02x}{:02x}{:02x}{:02x}",
                        ((self.0 >> 88) & 0xff) as u8,
                        ((self.0 >> 80) & 0xff) as u8,
                        ((self.0 >> 72) & 0xff) as u8,
                        ((self.0 >> 64) & 0xff) as u8,
                    ),
                    "}",
                )
            },
        };

        write!(fmt, "{s}")
    }
}
