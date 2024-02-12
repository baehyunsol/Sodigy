// TODO: remove this file when `lib.rs` works perfectly

#![deny(unused_imports)]

use hmath::BigInt;

mod error;
mod fmt;

pub use error::NumericParseError;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SodigyNumber {
    // since this variant is very rarely used, it's much more efficient to
    // reduce the size of `SodigyNumber` by using `Box`, than storing `BigNumber` directly
    Big(Box<BigNumber>),

    // an integer 0 ~ 2^64
    SmallInt(u64),

    // it's converted from BigNumber
    // for a number `A * 10^B` (A and B are both integers, 0 <= A < 2^48, -2^15 <= B < 2^15),
    // the u64 value is `A * 2^16 + B + 2^15`
    //
    // it uses decimal rather than binary because the numeric literals in Sodigy code uses decimals
    SmallRatio(u64),
}

impl SodigyNumber {
    pub fn is_zero(&self) -> bool {
        match self {
            SodigyNumber::Big(n) => n.is_zero(),
            SodigyNumber::SmallInt(n) => *n == 0,
            SodigyNumber::SmallRatio(n) => *n >> 16 == 0,
        }
    }

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
                let exp = (*n & 0xffff) as i64 - 32768;
                let digits = *n >> 16;

                (
                    digits.to_string().as_bytes().to_vec(),
                    exp,
                )
            },
        }
    }

    pub fn get_denom_and_numer(&self) -> (SodigyNumber, SodigyNumber) {  // (denom, numer)
        match self {
            SodigyNumber::Big(n) => n.get_denom_and_numer(),
            SodigyNumber::SmallInt(n) => (
                SodigyNumber::SmallInt(1),
                SodigyNumber::SmallInt(*n),
            ),
            SodigyNumber::SmallRatio(n) => {
                // self = digit * 10^exp
                let exp = (*n & 0xffff) as i64 - 32768;
                let mut digits = *n >> 16;

                // self = digits * 2^twos * 5^fives
                let mut twos = exp;
                let mut fives = exp;

                while digits & 1 == 0 {
                    digits >>= 1;
                    twos += 1;
                }

                while digits % 5 == 0 {
                    digits /= 5;
                    fives += 1;
                }

                let mut numer = digits as u128;
                let mut denom = 1u128;

                while twos < 0 {
                    denom <<= 1;
                    twos += 1;

                    if denom >= (1 << 64) {
                        return get_denom_and_numer_fallback(denom, numer, twos, fives);
                    }
                }

                while twos > 0 {
                    numer <<= 1;
                    twos -= 1;

                    if numer >= (1 << 64) {
                        return get_denom_and_numer_fallback(denom, numer, twos, fives);
                    }
                }

                while fives < 0 {
                    denom *= 5;
                    fives += 1;

                    if denom >= (1 << 64) {
                        return get_denom_and_numer_fallback(denom, numer, twos, fives);
                    }
                }

                while fives > 0 {
                    numer *= 5;
                    fives -= 1;

                    if numer >= (1 << 64) {
                        return get_denom_and_numer_fallback(denom, numer, twos, fives);
                    }
                }

                (
                    SodigyNumber::SmallInt(denom as u64),
                    SodigyNumber::SmallInt(numer as u64),
                )
            },
        }
    }

    // unfortunate that `SodigyNumber` is unsigned
    pub fn minus_one(n: Self, is_negative: bool) -> (Self, /* is_negative */ bool) {
        debug_assert!(n.is_integer());

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
            SodigyNumber::Big(n) if n.is_integer => {
                let mut n = n.to_hmath_bi();

                if is_negative {
                    n.neg_mut();
                }

                n.sub_i32_mut(1);
                let is_neg = n.is_neg();

                if is_neg {
                    n.neg_mut();
                }

                (
                    SodigyNumber::Big(Box::new(BigNumber::from_hmath_bi(&n))),
                    is_neg,
                )
            },
            _ => unreachable!(),
        }
    }

    pub fn gt(&self, other: &Self) -> bool {
        match (self, other) {
            (SodigyNumber::SmallInt(m), SodigyNumber::SmallInt(n)) => *m > *n,
            (SodigyNumber::SmallRatio(m), SodigyNumber::SmallRatio(n)) => {
                let exp1 = *m & 0xffff;
                let exp2 = *n & 0xffff;
                let mut digits1 = *m >> 16;
                let mut digits2 = *n >> 16;

                if digits1 == 0 {
                    return false;
                }

                else if digits2 == 0 {
                    return true;
                }

                let log10_d1 = log10(digits1);
                let log10_d2 = log10(digits2);

                if exp1 + log10_d1 != exp2 + log10_d2 {
                    return exp1 + log10_d1 > exp2 + log10_d2;
                }

                // let's compare first 15 digits
                digits1 *= pow10(16 - log10_d1);
                digits2 *= pow10(16 - log10_d2);

                digits1 > digits2
            },
            _ => todo!(),
        }
    }
}

// returns (x, y) where
// y / x = numer * 2^twos * 5^fives / denom
fn get_denom_and_numer_fallback(denom: u128, numer: u128, twos: i64, fives: i64) -> (SodigyNumber, SodigyNumber) {
    todo!()
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
        if -32768 <= self.exp && self.exp < 32768 && !self.is_integer {
            // `BigNumber::from_string` already filtered out invalid utf8 strings
            let digits = String::from_utf8(self.digits.clone()).unwrap();

            match digits.parse::<u64>() {
                Ok(n) if n < (1 << 48) => {
                    Some((n << 16) + (self.exp + 32768) as u64)
                },
                _ => None,
            }
        }

        else {
            None
        }
    }

    pub fn is_zero(&self) -> bool {
        self.digits == b"0"
    }

    pub fn get_denom_and_numer(&self) -> (SodigyNumber, SodigyNumber) {
        if self.is_integer {(
            SodigyNumber::SmallInt(1),
            SodigyNumber::Big(Box::new(self.clone())),
        )}

        else {
            todo!()
        }
    }

    pub fn to_hmath_bi(&self) -> BigInt {
        BigInt::from_string(
            unsafe { &String::from_utf8_unchecked(
                self.digits.iter().chain(
                    std::iter::repeat(&b'0').take(self.exp as usize)
                ).map(|c| *c).collect()
            ) }
        ).unwrap()
    }

    pub fn from_hmath_bi(n: &BigInt) -> Self {
        let s = n.to_string();

        // this branch is unreachable, tho
        if s == "0" {
            return BigNumber {
                digits: vec![b'0'],
                exp: 0,
                is_integer: true,
            };
        }

        let mut digits = s.into_bytes();
        let mut exp = 0;

        while digits.last() == Some(&b'0') {
            exp += 1;
            digits.pop().unwrap();
        }

        BigNumber {
            digits,
            exp,
            is_integer: true,
        }
    }
}

fn log10(n: u64) -> u64 {
    debug_assert!(n != 0);

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
            ("1230.0", "123", 1, false),
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

    #[test]
    fn gt_test() {
        let samples = vec![
            ("1.5", "1.25"),
            ("1.64", "1.2"),
            ("7.9", "1.23"),
            ("17e100", "101e99"),

            // TODO
            // ("1.0", "0"),
        ];

        // always a > b
        for (a, b) in samples.into_iter() {
            let a = SodigyNumber::from_string(a.as_bytes()).unwrap();
            let b = SodigyNumber::from_string(b.as_bytes()).unwrap();

            assert!(a.gt(&b));
        }
    }

    #[test]
    fn denom_numer_test() {
        let samples = [
            ("1.2", 5, 6),
            ("1.02", 50, 51),
            ("10.2", 5, 51),
            ("1.75", 4, 7),
            ("17.5", 2, 35),
            ("3.14", 50, 157),
        ];

        for (s, denom_, numer_) in samples.into_iter() {
            let n = BigNumber::from_string(s.as_bytes()).unwrap();
            let n = SodigyNumber::SmallRatio(n.try_into_u64().unwrap());
            let (denom, numer) = n.get_denom_and_numer();

            assert_eq!(SodigyNumber::SmallInt(denom_), denom);
            assert_eq!(SodigyNumber::SmallInt(numer_), numer);
        }
    }
}
