use super::SodigyNumber;
use hmath::Ratio;
use std::fmt;

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

#[cfg(test)]
mod tests {
    use crate::SodigyNumber;

    #[test]
    fn sodigy_number_fmt_test() {
        let samples = vec![
            "0", "-0", "0.0", "-0.0",
            "1", "2", "3", "4",
            "-1", "-2", "-3", "-4",
            "1.0", "2.0", "3.0", "4.0",
            "-1.0", "-2.0", "-3.0", "-4.0",
            "3e5", "3e20", "3.0e5", "3.0e20",
            "3e-5", "3e-20", "3.0e-5", "3.0e-20",
            "-3e5", "-3e20", "-3.0e5", "-3.0e20",
            "-3e-5", "-3e-20", "-3.0e-5", "-3.0e-20",
        ];

        for sample in samples.into_iter() {
            let is_integer = !(sample.contains(".") || sample.contains("e"));
            let n1 = SodigyNumber::from_string(sample.as_bytes());
            let s1 = n1.to_string();
            let n2 = SodigyNumber::from_string(s1.as_bytes());
            let s2 = n2.to_string();
            let n3 = SodigyNumber::from_string(s2.as_bytes());
            let s3 = n3.to_string();

            assert_eq!(s2, s3);

            if !is_integer {
                assert!(sample.contains(".") || sample.contains("e"));
            }
        }
    }
}
