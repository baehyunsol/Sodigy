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

            match (numer.len(), denom.len()) {
                (_, 1) if denom[0] == 1 => match numer.len() {
                    0 => unreachable!(),
                    1 | 2 => {
                        let n: u64 = numer[0] as u64 | ((*numer.get(1).unwrap_or(&0) as u64) << 32);

                        match i64::try_from(n) {
                            Ok(n) => InternedNumber {
                                value: InternedNumberValue::SmallInt(n),
                                is_integer,
                            },
                            Err(_) => InternedNumber {
                                value: InternedNumberValue::BigInt(BigInt {
                                    is_neg: false,
                                    nums: numer,
                                }),
                                is_integer,
                            },
                        }
                    },
                    3.. => InternedNumber {
                        value: InternedNumberValue::BigInt(BigInt {
                            is_neg: false,
                            nums: numer,
                        }),
                        is_integer,
                    },
                },
                (1 | 2, 1 | 2) => {
                    let numer_n: u64 = numer[0] as u64 | ((*numer.get(1).unwrap_or(&0) as u64) << 32);
                    let denom_n: u64 = denom[0] as u64 | ((*denom.get(1).unwrap_or(&0) as u64) << 32);

                    match i64::try_from(numer_n) {
                        Ok(numer) => InternedNumber {
                            value: InternedNumberValue::SmallRatio { numer, denom: denom_n },
                            is_integer,
                        },
                        Err(_) => InternedNumber {
                            value: InternedNumberValue::BigRatio(Ratio {
                                numer: BigInt { is_neg: false, nums: numer },
                                denom: BigInt { is_neg: false, nums: denom },
                            }),
                            is_integer,
                        },
                    }
                },
                _ => InternedNumber {
                    value: InternedNumberValue::BigRatio(Ratio {
                        numer: BigInt { is_neg: false, nums: numer },
                        denom: BigInt { is_neg: false, nums: denom },
                    }),
                    is_integer,
                },
            }
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
