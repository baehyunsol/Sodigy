#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Base {
    Hexadecimal,
    Decimal,
    Octal,
    Binary,
}

impl Base {
    pub fn is_valid_digit(&self, b: u8) -> bool {
        match (self, b) {
            (Base::Hexadecimal, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F') => true,
            (Base::Decimal, b'0'..=b'9') => true,
            (Base::Octal, b'0'..=b'7') => true,
            (Base::Binary, b'0' | b'1') => true,
            _ => false,
        }
    }

    pub fn invalid_digit_error_message(&self, b: u8) -> String {
        format!(
            "`{}` is not a valid digit for a {} number. Valid digits are {}.",
            b as char,
            format!("{self:?}").to_lowercase(),
            match self {
                Base::Hexadecimal => "0, 1, 2, 3, 4, 5, 6, 7, 8, 9, a, b, c, d, e and f",
                Base::Decimal => "0, 1, 2, 3, 4, 5, 6, 7, 8 and 9",
                Base::Octal => "0, 1, 2, 3, 4, 5, 6 and 7",
                Base::Binary => "0 and 1",
            },
        )
    }
}
