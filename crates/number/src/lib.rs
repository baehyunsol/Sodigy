mod base;
mod big_int;
mod endec;
mod error;
mod ratio;

pub use base::Base;
pub use big_int::{
    BigInt,
    cmp::*,
    op::*,
};
pub(crate) use error::ParseIntError;
pub use ratio::Ratio;

// `InternedString` implements `Copy` (hence "interned"), but
// `InternedNumber` doesn't. My idea is that strings, including identifiers
// are used really frequently by the compiler, but `BigInt`s and `BigRatio`s
// are used less frequently, so it's okay to use heap memory.
#[derive(Clone, Debug)]
pub struct InternedNumber {
    pub value: InternedNumberValue,

    // It remembers the original literal.
    // For example, `1.0` and `1` has the same `value` but different `is_integer`.
    pub is_integer: bool,
}

#[derive(Clone, Debug)]
pub enum InternedNumberValue {
    SmallInt(i64),
    SmallRatio {
        numer: i64,
        denom: u64,
    },
    BigInt(BigInt),
    BigRatio(Ratio),
}

impl InternedNumber {
    pub fn from_u32(n: u32, is_integer: bool) -> Self {
        InternedNumber {
            value: InternedNumberValue::SmallInt(n as i64),
            is_integer,
        }
    }

    pub fn negate_mut(&mut self) {
        match &mut self.value {
            InternedNumberValue::SmallInt(n) => match n.checked_neg() {
                Some(nn) => {
                    *n = nn;
                },
                None => todo!(),
            },
            InternedNumberValue::SmallRatio { numer, .. } => match numer.checked_neg() {
                Some(nn) => {
                    *numer = nn;
                },
                None => todo!(),
            },
            _ => todo!(),
        }
    }
}

/// Lexer must guarantee that it's parse-able.
pub fn intern_number(
    base: Base,
    integer: &[u8],

    // `frac` is always decimal
    frac: &[u8],
    exp: i64,

    // of the original literal
    is_integer: bool,
) -> InternedNumber {
    match (base, frac.len(), exp) {
        (Base::Hexadecimal, 0, 0) => match i64::from_str_radix(&String::from_utf8_lossy(integer), 16) {
            Ok(n) => InternedNumber {
                value: InternedNumberValue::SmallInt(n),
                is_integer,
            },
            Err(_) => InternedNumber {
                value: InternedNumberValue::BigInt(BigInt::parse_positive_hex(integer).unwrap()),
                is_integer,
            },
        },
        (Base::Hexadecimal, _, _) => unreachable!(),
        (Base::Decimal, 0, 0) => match String::from_utf8_lossy(integer).parse::<i64>() {
            Ok(n) => InternedNumber {
                value: InternedNumberValue::SmallInt(n),
                is_integer,
            },
            Err(_) => InternedNumber {
                value: InternedNumberValue::BigInt(BigInt::parse_positive_decimal(integer).unwrap()),
                is_integer,
            },
        },
        (Base::Decimal, 0, _) => match String::from_utf8_lossy(integer).parse::<u64>() {
            Ok(integer) => match exp {
                0..0xffff_ffff => match 10u64.checked_pow(exp as u32) {
                    Some(exp) => match integer.checked_mul(exp) {
                        Some(n) => match i64::try_from(n) {
                            Ok(n) => InternedNumber {
                                value: InternedNumberValue::SmallInt(n),
                                is_integer,
                            },
                            Err(_) => todo!(),
                        },
                        None => todo!(),
                    },
                    None => todo!(),
                },
                _ => todo!(),
            },
            Err(e) => todo!(),
        },
        (Base::Decimal, _, 0) => match String::from_utf8_lossy(integer).parse::<u64>() {
            Ok(int_numer) => {
                let mut frac_vec = frac.to_vec();

                // 1.0 -> 1
                // 1.500 -> 1.5
                while let Some(b'0') = frac_vec.last() {
                    frac_vec.pop();
                }

                match frac_vec.len() {
                    0 => match i64::try_from(int_numer) {
                        Ok(n) => InternedNumber {
                            value: InternedNumberValue::SmallInt(n),
                            is_integer,
                        },
                        Err(_) => intern_number(base, integer, b"", 0, is_integer),
                    },
                    1..=16 => {
                        let mut frac_numer = String::from_utf8_lossy(&frac_vec).parse::<u64>().unwrap();
                        let mut frac_denom = 10u64.pow(frac_vec.len() as u32);
                        let r = gcd(frac_numer, frac_denom);
                        frac_numer /= r;
                        frac_denom /= r;

                        // n = (int_numer * frac_denom + frac_numer) / frac_denom
                        match int_numer.checked_mul(frac_denom) {
                            Some(int_numer) => match int_numer.checked_add(frac_numer) {
                                Some(numer) => match i64::try_from(numer) {
                                    Ok(numer) => InternedNumber {
                                        value: InternedNumberValue::SmallRatio {
                                            numer,
                                            denom: frac_denom,
                                        },
                                        is_integer,
                                    },
                                    Err(_) => todo!(),
                                },
                                None => todo!(),
                            },
                            None => todo!(),
                        }
                    },
                    17.. => todo!(),
                }
            },
            Err(_) => panic!("TODO: (base: {base:?}, int: {:?}, frac: {:?}, exp: {exp:?})", String::from_utf8_lossy(integer), String::from_utf8_lossy(frac)),
        },
        (Base::Decimal, _, _) => panic!("TODO: (base: {base:?}, int: {:?}, frac: {:?}, exp: {exp:?})", String::from_utf8_lossy(integer), String::from_utf8_lossy(frac)),
        (Base::Octal, 0, 0) => match i64::from_str_radix(&String::from_utf8_lossy(integer), 8) {
            Ok(n) => InternedNumber {
                value: InternedNumberValue::SmallInt(n),
                is_integer,
            },
            Err(_) => todo!(),
        },
        (Base::Octal, _, _) => unreachable!(),
        (Base::Binary, 0, 0) => match i64::from_str_radix(&String::from_utf8_lossy(integer), 2) {
            Ok(n) => InternedNumber {
                value: InternedNumberValue::SmallInt(n),
                is_integer,
            },
            Err(_) => todo!(),
        },
        (Base::Binary, _, _) => unreachable!(),
    }
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    }

    else {
        gcd(b, a % b)
    }
}
