#![deny(unused_imports)]

use hmath::{BigInt, Ratio};

mod fmt;

// SodigyNumber representation must be unique. If the same numeric literal
// can be converted to multiple variants, that's a bug.
// It's typed. That means literal `0` and `0.0` are different. The first one
// is `SmallInt`, while the second one is `SmallRatio`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SodigyNumber {
    BigInt(Box<BigInt>),
    BigRatio(Box<Ratio>),
    SmallInt(i64),

    // 1. `denom` is always positive
    // 2. if `numer` is 0, `denom` is 1
    // 3. otherwise, gcd(numer, denom) is always 1
    SmallRatio { numer: i32, denom: u32 },
}

impl SodigyNumber {
    pub fn is_zero(&self) -> bool {
        match self {
            SodigyNumber::BigInt(n) => n.is_zero(),
            SodigyNumber::BigRatio(n) => n.is_zero(),
            SodigyNumber::SmallInt(n) => *n == 0,
            SodigyNumber::SmallRatio { numer, .. } => *numer == 0,
        }
    }

    // `s` is guaranteed to be a valid, decimal number. `s` may contain `e` or a decimal separator.
    pub fn from_string(s: &[u8]) -> Self {
        let s = String::from_utf8(s.to_vec()).unwrap();

        if let Ok(n) = s.parse::<i64>() {
            SodigyNumber::SmallInt(n)
        }

        else if let Ok(n) = BigInt::from_string(&s) {
            SodigyNumber::BigInt(Box::new(n))
        }

        else {
            let n = Ratio::from_string(&s).unwrap();

            // As far as i know, this is the only way to check the size
            // of denom and numer without cloning it
            let (denom, denom_neg, numer, numer_neg) = n.into_raw();

            if denom.len() == 1 && numer.len() == 1 {
                // we have to be very careful to avoid overflows
                if numer_neg {
                    let numer_i64 = -(numer[0] as i64);

                    if let Ok(numer) = i32::try_from(numer_i64) {
                        SodigyNumber::SmallRatio {
                            denom: denom[0],
                            numer,
                        }
                    }

                    else {
                        SodigyNumber::BigRatio(
                            Box::new(Ratio::from_raw(denom, denom_neg, numer, numer_neg))
                        )
                    }
                }

                else if let Ok(numer) = i32::try_from(numer[0]) {
                    SodigyNumber::SmallRatio {
                        denom: denom[0],
                        numer,
                    }
                }

                else {
                    SodigyNumber::BigRatio(
                        Box::new(Ratio::from_raw(denom, denom_neg, numer, numer_neg))
                    )
                }
            }

            else {
                SodigyNumber::BigRatio(
                    Box::new(Ratio::from_raw(denom, denom_neg, numer, numer_neg))
                )
            }
        }
    }

    pub fn is_integer(&self) -> bool {
        match self {
            SodigyNumber::BigInt(_)
            | SodigyNumber::SmallInt(_) => true,
            SodigyNumber::BigRatio(_)
            | SodigyNumber::SmallRatio { .. } => false,
        }
    }

    /// returns `(digits, exp)` where `self = digits * 10^exp`.
    /// same value might return different results (eg. (3, 1) and (30, 0)).
    pub fn digits_and_exp(&self) -> (Vec<u8>, i64) {
        match self {
            // the original implementation is unsigned, so returning `(Vec<u8>, i64)`
            // makes sense. but now that it's signed, we need some other return type...
            _ => todo!(),
        }
    }

    pub fn get_denom_and_numer(&self) -> (SodigyNumber, SodigyNumber) {  // (denom, numer)
        match self {
            SodigyNumber::BigInt(n) => (
                SodigyNumber::SmallInt(1),
                SodigyNumber::BigInt(n.clone()),
            ),
            SodigyNumber::BigRatio(n) => {
                let denom = n.get_denom();
                let numer = n.get_numer();

                let denom = if let Ok(denom) = i64::try_from(&denom) {
                    SodigyNumber::SmallInt(denom)
                } else {
                    SodigyNumber::BigInt(Box::new(denom.clone()))
                };
                let numer = if let Ok(numer) = i64::try_from(&numer) {
                    SodigyNumber::SmallInt(numer)
                } else {
                    SodigyNumber::BigInt(Box::new(numer.clone()))
                };

                (denom, numer)
            },
            SodigyNumber::SmallInt(n) => (
                SodigyNumber::SmallInt(1),
                SodigyNumber::SmallInt(*n),
            ),
            SodigyNumber::SmallRatio { denom, numer } => (
                SodigyNumber::SmallInt(*denom as i64),
                SodigyNumber::SmallInt(*numer as i64),
            ),
        }
    }

    pub fn minus_one(&self) -> Self {
        debug_assert!(self.is_integer());

        match self {
            SodigyNumber::BigInt(n) => SodigyNumber::BigInt(Box::new(
                n.sub_i32(1)
            )),
            SodigyNumber::SmallInt(n) => match n.checked_sub(1) {
                Some(n) => SodigyNumber::SmallInt(n),
                None => {
                    let mut n: BigInt = n.into();
                    n.sub_i32_mut(1);

                    SodigyNumber::BigInt(Box::new(n))
                },
            },
            _ => unreachable!(),
        }
    }

    pub fn gt(&self, other: &Self) -> bool {
        match (self, other) {
            (SodigyNumber::BigInt(m), SodigyNumber::BigInt(n)) => m.gt(n),
            (SodigyNumber::BigRatio(m), SodigyNumber::BigRatio(n)) => m.gt(n),
            (SodigyNumber::SmallInt(m), SodigyNumber::SmallInt(n)) => *m > *n,
            (
                SodigyNumber::SmallRatio { denom: denom1, numer: numer1 },
                SodigyNumber::SmallRatio { denom: denom2, numer: numer2 },
            ) => {
                // n1 / d1 > n2 / d2 -> n1 * d2 > n2 * d1

                *numer1 as i64 * *denom2 as i64 > *numer2 as i64 * *denom1 as i64
            },
            _ => todo!(),
        }
    }

    pub fn neg(&self) -> Self {
        match self {
            SodigyNumber::BigInt(n) => SodigyNumber::BigInt(Box::new(n.neg())),
            SodigyNumber::BigRatio(n) => SodigyNumber::BigRatio(Box::new(n.neg())),
            SodigyNumber::SmallInt(n) => match n.checked_neg() {
                Some(n) => SodigyNumber::SmallInt(n),
                None => SodigyNumber::BigInt(Box::new(
                    BigInt::from(*n).neg()
                )),
            },
            SodigyNumber::SmallRatio { denom, numer } => match numer.checked_neg() {
                Some(numer) => SodigyNumber::SmallRatio {
                    denom: *denom,
                    numer,
                },
                None => SodigyNumber::BigRatio(Box::new(Ratio::from_denom_and_numer(
                    BigInt::from(*denom),
                    BigInt::from(*numer).neg(),
                )))
            }
        }
    }
}

impl From<u32> for SodigyNumber {
    fn from(n: u32) -> Self {
        SodigyNumber::SmallInt(n as i64)
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
            // Do not convert `SmallRatio` into `u32`: integers and ratios are different
            _ => Err(()),
        }
    }
}
