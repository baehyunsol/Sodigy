#![deny(unused_imports)]

mod fmt;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Keyword {
    Let = 0,
    Enum = 1,
    Struct = 2,
    Module = 3,
    Import = 4,
    As = 5,
    From = 6,
    If = 7,
    Else = 8,
    Pattern = 9,
    Match = 10,
}

impl Keyword {
    pub fn to_utf8(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}

pub const fn keywords() -> [Keyword; 11] {
    [
        Keyword::Let,
        Keyword::Enum,
        Keyword::Struct,
        Keyword::Module,
        Keyword::Import,
        Keyword::As,
        Keyword::From,
        Keyword::If,
        Keyword::Else,
        Keyword::Pattern,
        Keyword::Match,
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn keywords_all() {
        let keywords = super::keywords();

        for (i, k) in keywords.into_iter().enumerate() {
            assert_eq!(i, k as usize);
        }
    }
}
