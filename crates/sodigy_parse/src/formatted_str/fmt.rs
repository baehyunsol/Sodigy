use super::FormattedStringElement;
use std::fmt;

impl fmt::Debug for FormattedStringElement {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}",
            match self {
                FormattedStringElement::Value(v) => format!("Value({v:?})"),
                FormattedStringElement::Literal(s) => format!("String({:?})", s),
            },
        )
    }
}
