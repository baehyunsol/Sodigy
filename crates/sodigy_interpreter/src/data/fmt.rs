use super::SodigyData;
use crate::{to_rust_string, to_string};
use std::fmt;

impl fmt::Display for SodigyData {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = if let Ok(s) = to_string(self) {
            if let Ok(s) = to_rust_string(s.as_ref()) {
                s.iter().map(|c| char::from_u32(*c).unwrap()).collect()
            }

            else {
                String::from("___Error")
            }
        }

        else {
            String::from("__Error")
        };

        write!(fmt, "{s}")
    }
}

