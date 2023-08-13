use super::RangeType;
use crate::session::{InternedString, LocalParseSession};
use hmath::Ratio;

#[derive(Clone, PartialEq)]
pub enum PatternErrorKind {
    /// `1.5 .. 2.0`
    NonIntegerInRange(Ratio),

    /// 5..3
    InvalidIntegerRange(Ratio, Ratio, RangeType),

    /// "z".."a"
    InvalidCharRange(u32, u32, RangeType),

    /// ($a, .., $b, .., $c)
    MultipleShorthands,

    /// Foo { a, a }\
    /// ($a, $a)
    MultiFieldBindingInPattern(InternedString),
}

impl PatternErrorKind {
    pub fn render_err(&self, session: &LocalParseSession) -> String {
        match self {
            PatternErrorKind::NonIntegerInRange(n) => format!(
                "expected an integer, found {n}",
            ),
            PatternErrorKind::InvalidIntegerRange(from, to, rt) => format!(
                "{from}{rt}{to} is an invalid range",
            ),
            PatternErrorKind::InvalidCharRange(from, to, rt) => format!(
                "{:?}{rt}{:?} is an invalid range",
                char::from_u32(*from).expect("Internal Compiler Error E044D34D6FE"),
                char::from_u32(*to).expect("Internal Compiler Error 0F80EE9FADE"),
            ),
            PatternErrorKind::MultipleShorthands => String::from("`..` can only be used once per pattern"),
            PatternErrorKind::MultiFieldBindingInPattern(name) => format!(
                "field `{}` bound multiple times in the pattern",
                name.to_string(session),
            ),
        }
    }
}
