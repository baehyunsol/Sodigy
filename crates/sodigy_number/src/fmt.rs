use super::SodigyNumber;
use sodigy_test::sodigy_assert_eq;
use std::fmt;

impl fmt::Debug for SodigyNumber {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}{}{}",
            String::from_utf8_lossy(&self.digits).to_string(),
            if self.is_integer { "" } else { "." },
            if self.exp == 0 { String::new() } else { format!("e{}", self.exp) },
        )
    }
}

impl fmt::Display for SodigyNumber {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut digits = self.digits.clone();
        let mut exp = self.exp;

        if self.is_integer {
            while exp > 0 {
                exp -= 1;
                digits.push(b'0');
            }

            while exp < 0 {
                sodigy_assert_eq!(digits.last(), Some(&b'0'));

                exp += 1;
                digits.pop().unwrap();
            }
        }

        write!(
            fmt,
            "{}{}{}",
            String::from_utf8_lossy(&digits).to_string(),
            if self.is_integer { "" } else { "." },
            if exp == 0 { String::new() } else { format!("e{}", exp) },
        )
    }
}
