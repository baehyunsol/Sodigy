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
pub use ratio::{Ratio, op::*};

// `InternedString` implements `Copy` (hence "interned"), but
// `InternedNumber` doesn't. My idea is that strings, including identifiers
// are used really frequently by the compiler, but `BigInt`s and `BigRatio`s
// are used less frequently, so it's okay to use heap memory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InternedNumber {
    pub value: InternedNumberValue,

    // It remembers the original literal.
    // For example, `1.0` and `1` has the same `value` but different `is_integer`.
    // When doing comptime-eval, this field acts like a type-information.
    pub is_integer: bool,
}

// Caution: A number must be represented in a most efficient way.
// If it can be represented in `SmallInt`, it must be.
// Else if it can be represented in `BigInt`, it must be.
// Else if it can be represented in `SmallRatio`, it must be.
// Otherwise, it must be represented in `BigRatio`.
// It's safer to use `intern_number` than constructing this
// value manually.
#[derive(Clone, Debug, Eq, PartialEq)]
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

impl InternedNumberValue {
    pub fn is_zero(&self) -> bool {
        matches!(self, InternedNumberValue::SmallInt(0))
    }
}

pub fn unintern_number(n: InternedNumberValue) -> Ratio {
    match n {
        InternedNumberValue::SmallInt(n) => Ratio { numer: BigInt::from(n), denom: BigInt::one() },
        InternedNumberValue::SmallRatio { numer, denom } => Ratio { numer: BigInt::from(numer), denom: BigInt::from(denom) },
        InternedNumberValue::BigInt(n) => Ratio { numer: n, denom: BigInt::one() },
        InternedNumberValue::BigRatio(n) => n,
    }
}

pub fn intern_number(n: Ratio) -> InternedNumberValue {
    if n.denom.is_one() {
        match i64::try_from(&n.numer) {
            Ok(n) => InternedNumberValue::SmallInt(n),
            Err(()) => InternedNumberValue::BigInt(n.numer),
        }
    }

    else {
        match (i64::try_from(&n.numer), u64::try_from(&n.denom)) {
            (Ok(numer), Ok(denom)) => InternedNumberValue::SmallRatio { numer, denom },
            _ => InternedNumberValue::BigRatio(n),
        }
    }
}

/// Lexer must guarantee that it's parse-able.
pub fn intern_number_raw(
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
        (Base::Decimal, _, _) => {
            let mut integer = BigInt::parse_positive_decimal(integer).unwrap();
            let mut frac_numer = match frac.len() {
                0 => BigInt::zero(),
                _ => BigInt::parse_positive_decimal(frac).unwrap(),
            };
            let mut frac_denom = {
                let fds = format!("1{}", "0".repeat(frac.len()));
                BigInt::parse_positive_decimal(fds.as_bytes()).unwrap()
            };

            let r = gcd_ubi(&frac_numer.nums, &frac_denom.nums);
            frac_numer.nums = div_ubi(&frac_numer.nums, &r);
            frac_denom.nums = div_ubi(&frac_denom.nums, &r);

            match exp {
                ..0 => {
                    let power = todo!();  // 10^(-exp)
                    frac_denom.nums = mul_ubi(&frac_denom.nums, power);
                },
                0 => {},
                1.. => {
                    let power = todo!();  // 10^exp
                    integer.nums = mul_ubi(&integer.nums, power);
                    frac_numer.nums = mul_ubi(&frac_numer.nums, power);
                },
            }

            let mut numer = add_ubi(&mul_ubi(&integer.nums, &frac_denom.nums), &frac_numer.nums);
            let mut denom = frac_denom.nums;

            let r = gcd_ubi(&numer, &denom);
            numer = div_ubi(&numer, &r);
            denom = div_ubi(&denom, &r);

            let value = intern_number(Ratio {
                numer: BigInt { is_neg: false, nums: numer },
                denom: BigInt { is_neg: false, nums: denom },
            });
            InternedNumber { value, is_integer }
        },
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
