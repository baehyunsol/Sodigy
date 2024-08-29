use super::PathOrRawInput;
use sodigy_error::trim_long_string;
use std::fmt;

impl<'a> fmt::Debug for PathOrRawInput<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt, "{}",
            match self {
                PathOrRawInput::Path(p) => format!(
                    "Path({})",
                    trim_long_string(format!("{p:?}"), 64, 64),
                ),
                PathOrRawInput::RawInput(raw_input) => format!(
                    "RawInput({})",
                    trim_long_string(String::from_utf8_lossy(raw_input).to_string(), 32, 32),
                ),
            },
        )
    }
}
