use sodigy_span::RenderableSpan;

mod kind;
mod level;
mod token;
mod warning;

pub use kind::ErrorKind;
pub use level::ErrorLevel;
pub use token::ErrorToken;
pub use warning::{Warning, WarningKind};

#[derive(Clone, Debug)]
pub struct Error {
    pub kind: ErrorKind,

    // errors are sorted by e.spans[0].span
    // the span renderer will try to render non-auxiliary spans first
    pub spans: Vec<RenderableSpan>,

    pub note: Option<String>,
}

impl Default for Error {
    fn default() -> Error {
        Error {
            // please don't use this value
            kind: ErrorKind::InvalidUtf8,
            spans: vec![],
            note: None,
        }
    }
}

// I defined it here because it's usually for error messages.
pub fn to_ordinal(n: usize) -> String {
    match n {
        _ if n % 10 == 1 && n != 11 => format!("{n}st"),
        _ if n % 10 == 2 && n != 12 => format!("{n}nd"),
        _ if n % 10 == 3 && n != 13 => format!("{n}rd"),
        _ => format!("{n}th"),
    }
}
