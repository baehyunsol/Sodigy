#[derive(Clone, Copy)]
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
}
