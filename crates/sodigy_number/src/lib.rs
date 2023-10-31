mod err;
mod fmt;

pub use err::NumericParseError;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SodigyNumber {
    Big(BigNumber),
    Small(u64),
}

impl SodigyNumber {
    // `s` is guaranteed to be a valid, decimal number. `s` may contain `e` or a decimal separator.
    pub fn from_string(s: &[u8]) -> Result<Self, NumericParseError> {
        if s.len() < 16 {
            if let Ok(s) = String::from_utf8(s.to_vec()) {
                if let Ok(n) = s.parse::<u64>() {
                    return Ok(SodigyNumber::Small(n));
                }
            }
        }

        Ok(SodigyNumber::Big(BigNumber::from_string(s)?))
    }

    pub fn is_integer(&self) -> bool {
        match self {
            SodigyNumber::Big(n) => n.is_integer,
            SodigyNumber::Small(_) => true,
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct BigNumber {
    // it's in decimal
    digits: Vec<u8>,

    // exp 10
    exp: i64,

    pub is_integer: bool,
}

enum ParseState {
    Integer,
    Frac,
    Exp,
}

impl BigNumber {
    // `s` is guaranteed to be a valid, decimal number. `s` may contain `e` or a decimal separator.
    fn from_string(s: &[u8]) -> Result<Self, NumericParseError> {
        let mut digits = Vec::with_capacity(s.len());
        let mut exp_digits = vec![];
        let mut exp = 0;
        let mut curr_state = ParseState::Integer;
        let mut is_integer = true;

        for c in s.iter() {
            match curr_state {
                ParseState::Integer => {
                    if *c == b'.' {
                        curr_state = ParseState::Frac;
                        is_integer = false;
                    }

                    else if *c == b'e' || *c == b'E' {
                        curr_state = ParseState::Exp;
                        is_integer = false;
                    }

                    else {
                        digits.push(*c);
                    }
                },
                ParseState::Frac => {
                    if *c == b'e' || *c == b'E' {
                        curr_state = ParseState::Exp;
                    }

                    else {
                        exp -= 1;
                        digits.push(*c);
                    }
                },
                ParseState::Exp => {
                    exp_digits.push(*c);
                },
            }
        }

        let exp_digits = String::from_utf8(exp_digits).unwrap();

        let mut exp = match exp_digits.parse::<i64>() {
            _ if exp_digits.is_empty() => exp,
            Ok(n) => match n.checked_add(exp) {
                Some(n) => n,
                None => {
                    return Err(NumericParseError::ExpOverflow);
                },
            },
            Err(_) => {
                return Err(NumericParseError::ExpOverflow);
            }
        };

        let mut leading_zeros = 0;

        while leading_zeros < digits.len() - 1
        && digits[leading_zeros] == b'0' {
            leading_zeros += 1;
        }

        digits = digits[leading_zeros..].to_vec();

        while digits.last() == Some(&b'0')
        && digits.len() > 1
        && exp < i64::MAX {
            exp += 1;
            digits.pop().unwrap();
        }

        if digits == b"0" {
            exp = 0;
        }

        Ok(BigNumber {
            digits,
            exp,
            is_integer,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() {
        let samples = vec![
            ("123.456e10", "123456", 7, false),
            ("0.123", "123", -3, false),
            ("1230", "123", 1, true),
            ("0.0123", "123", -4, false),
            ("0.0123e3", "123", -1, false),
            ("0.0123e-3", "123", -7, false),
            ("0e10", "0", 0, false),
            ("0.0e-1", "0", 0, false),
        ];

        for (s, digits, exp, is_integer) in samples.into_iter() {
            assert_eq!(
                BigNumber::from_string(s.as_bytes()).unwrap(),
                BigNumber {
                    digits: digits.as_bytes().to_vec(),
                    exp,
                    is_integer,
                }
            );
        }
    }
}
