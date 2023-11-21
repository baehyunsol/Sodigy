#![deny(unused_imports)]

use sodigy_test::sodigy_assert;

mod err;
mod fmt;

pub use err::NumericParseError;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SodigyNumber {
    // since this variant is very rarely used, it's much more efficient to
    // reduce the size of `SodigyNumber` by using `Box`, than storing `BigNumber` directly
    Big(Box<BigNumber>),

    // an integer 0 ~ 18446744073709551615
    SmallInt(u64),

    // it's converted from BigNumber
    // for a number `A * 10^B` (A and B are both integers, 0 <= A < 281474976710655, -32768 <= B < 32768),
    // the u64 value is `A * 65536 + B + 32768`
    //
    // it uses decimal rather than binary because the numeric literals in Sodigy code uses decimals
    SmallRatio(u64),
}

impl SodigyNumber {
    // `s` is guaranteed to be a valid, decimal number. `s` may contain `e` or a decimal separator.
    pub fn from_string(s: &[u8]) -> Result<Self, NumericParseError> {
        if s.len() < 21 {
            if let Ok(s) = String::from_utf8(s.to_vec()) {
                if let Ok(n) = s.parse::<u64>() {
                    return Ok(SodigyNumber::SmallInt(n));
                }
            }
        }

        match BigNumber::from_string(s) {
            Ok(n) => match n.try_into_u64() {
                Some(n) => Ok(SodigyNumber::SmallRatio(n)),
                _ => Ok(SodigyNumber::Big(Box::new(n))),
            },
            Err(e) => Err(e),
        }
    }

    pub fn is_integer(&self) -> bool {
        match self {
            SodigyNumber::Big(n) => n.is_integer,
            SodigyNumber::SmallInt(_) => true,
            SodigyNumber::SmallRatio(_) => false,
        }
    }

    /// returns `(digits, exp)` where `self = digits * 10^exp`.
    /// same value might return different results (eg. (3, 1) and (30, 0)).
    pub fn digits_and_exp(&self) -> (Vec<u8>, i64) {
        match self {
            SodigyNumber::Big(n) => (n.digits.clone(), n.exp),
            SodigyNumber::SmallInt(n) => (
                n.to_string().as_bytes().to_vec(),
                0,
            ),
            SodigyNumber::SmallRatio(n) => {
                let exp = (*n % 65536) as i64 - 32768;
                let digits = *n / 65536;

                (
                    digits.to_string().as_bytes().to_vec(),
                    exp,
                )
            },
        }
    }

    // unfortunate that `SodigyNumber` is unsigned
    pub fn minus_one(n: Self, is_negative: bool) -> (Self, bool) {
        match n {
            SodigyNumber::SmallInt(n) => if is_negative {
                match n.checked_add(1) {
                    Some(n) => ((n as u32).into(), true),
                    _ => todo!(),
                }
            } else {
                if n == 0 {
                    (1.into(), true)
                }

                else {
                    (((n - 1) as u32).into(), false)
                }
            },
            _ => todo!(),
        }
    }

    pub fn gt(&self, other: &Self) -> bool {
        match (self, other) {
            (SodigyNumber::SmallInt(m), SodigyNumber::SmallInt(n)) => *m > *n,
            (SodigyNumber::SmallRatio(m), SodigyNumber::SmallRatio(n)) => {
                let exp1 = *m % 65536;
                let exp2 = *n % 65536;
                let digits1 = *m / 65536;
                let digits2 = *n / 65536;

                if digits1 == 0 {
                    return false;
                }

                else if digits2 == 0 {
                    return true;
                }

                // TODO: use pow10 and log10 defined below

                // we can't just compare `exp`s: the range of `digits`s vary
                todo!()
            },
            _ => todo!(),
        }
    }
}

impl From<u32> for SodigyNumber {
    fn from(n: u32) -> Self {
        SodigyNumber::SmallInt(n as u64)
    }
}

impl TryFrom<&SodigyNumber> for u32 {
    type Error = ();

    fn try_from(n: &SodigyNumber) -> Result<u32, ()> {
        match n {
            SodigyNumber::SmallInt(n) => match u32::try_from(*n) {
                Ok(n) => Ok(n),
                _ => Err(()),
            },
            _ => Err(())
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct BigNumber {
    // it's in decimal
    pub digits: Vec<u8>,

    // exp 10
    pub exp: i64,

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

    fn try_into_u64(&self) -> Option<u64> {
        if -32768 <= self.exp && self.exp < 32768 {
            // `BigNumber::from_string` already filtered out invalid utf8 strings
            let digits = String::from_utf8(self.digits.clone()).unwrap();

            match digits.parse::<u64>() {
                Ok(n) if n < 281474976710656 => {
                    Some(n * 65536 + (self.exp + 32768) as u64)
                },
                _ => None
            }
        }

        else {
            None
        }
    }
}

fn log10(n: u64) -> u64 {
    sodigy_assert!(n != 0);

    if n >= 10_000 {
        4 + log10(n / 10_000)
    }

    else if n >= 100 {
        if n >= 1_000 {
            3
        }

        else {
            2
        }
    }

    else {
        if n >= 10 {
            1
        }

        else {
            0
        }
    }
}

fn pow10(n: u64) -> u64 {
    if n < 8 {
        [
            1, 10, 100, 1_000,
            10_000, 100_000, 1_000_000,
            10_000_000,
        ][n as usize]
    }

    else {
        100_000_000 * pow10(n - 8)
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
