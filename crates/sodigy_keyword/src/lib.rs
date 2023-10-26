mod fmt;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Keyword {
    Def = 0,
    Enum = 1,
    Struct = 2,
    Module = 3,
    Use = 4,
    As = 5,
    If = 6,
    Else = 7,
    Let = 8,
    Match = 9,
}

impl Keyword {
    pub fn to_utf8(&self) -> Vec<u8> {
        format!("{}", self).as_bytes().to_vec()
    }
}

pub const fn keywords() -> [Keyword; 10] {
    [
        Keyword::Def,
        Keyword::Enum,
        Keyword::Struct,
        Keyword::Module,
        Keyword::Use,
        Keyword::As,
        Keyword::If,
        Keyword::Else,
        Keyword::Let,
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
