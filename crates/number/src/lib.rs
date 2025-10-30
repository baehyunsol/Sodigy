mod base;
mod endec;
mod fmt;

pub use base::Base;

#[derive(Clone, Copy)]
pub struct InternedNumber {
    pub value: InternedNumberValue,

    // It remembers the original literal.
    // For example, `1.0` and `1` has the same `value` but different `is_integer`.
    pub is_integer: bool,
}

#[derive(Clone, Copy)]
pub enum InternedNumberValue {
    SmallInteger(i64),
    SmallRatio {
        numer: i64,
        denom: u64,
    },
}

impl InternedNumber {
    pub fn from_u32(n: u32, is_integer: bool) -> Self {
        InternedNumber {
            value: InternedNumberValue::SmallInteger(n as i64),
            is_integer,
        }
    }
}

// impl From<u32> for InternedNumber {
//     fn from(n: u32) -> InternedNumber {
//         InternedNumber::SmallInteger(n as i64)
//     }
// }

/// Lexer must guarantee that it's parse-able.
pub fn intern_number(
    base: Base,
    integer: &[u8],
    frac: &[u8],

    // of the original literal
    is_integer: bool,
) -> InternedNumber {
    match (base, frac.len()) {
        (Base::Hexadecimal, 0) => match i64::from_str_radix(&String::from_utf8_lossy(integer), 16) {
            Ok(n) => InternedNumber {
                value: InternedNumberValue::SmallInteger(n),
                is_integer,
            },
            Err(_) => todo!(),
        },
        (Base::Hexadecimal, _) => unreachable!(),
        (Base::Decimal, 0) => match String::from_utf8_lossy(integer).parse::<i64>() {
            Ok(n) => InternedNumber {
                value: InternedNumberValue::SmallInteger(n),
                is_integer,
            },
            Err(_) => todo!(),
        },
        (Base::Decimal, _) => match String::from_utf8_lossy(integer).parse::<u64>() {
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
                            value: InternedNumberValue::SmallInteger(n),
                            is_integer,
                        },
                        Err(_) => intern_number(base, integer, b"", is_integer),
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
            Err(_) => panic!("TODO: (base: {base:?}, int: {:?}, frac: {:?})", String::from_utf8_lossy(integer), String::from_utf8_lossy(frac)),
        },
        (Base::Octal, 0) => match i64::from_str_radix(&String::from_utf8_lossy(integer), 8) {
            Ok(n) => InternedNumber {
                value: InternedNumberValue::SmallInteger(n),
                is_integer,
            },
            Err(_) => todo!(),
        },
        (Base::Octal, _) => unreachable!(),
        (Base::Binary, 0) => match i64::from_str_radix(&String::from_utf8_lossy(integer), 2) {
            Ok(n) => InternedNumber {
                value: InternedNumberValue::SmallInteger(n),
                is_integer,
            },
            Err(_) => todo!(),
        },
        (Base::Binary, _) => unreachable!(),
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
