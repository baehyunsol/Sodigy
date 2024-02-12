use super::SodigyNumber;
use hmath::Ratio;
use std::fmt;

// TODO: `3e5` has type SmallRatio, but it's rendered to `300000`. It has to be `300000.0`
impl fmt::Display for SodigyNumber {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            match self {
                SodigyNumber::BigInt(n) => n.to_string(),
                SodigyNumber::BigRatio(n) => n.to_string(),
                SodigyNumber::SmallInt(n) => n.to_string(),
                SodigyNumber::SmallRatio { denom, numer } => Ratio::from_denom_and_numer(
                    denom.into(),
                    numer.into(),
                ).to_string(),
            },
        )
    }
}
