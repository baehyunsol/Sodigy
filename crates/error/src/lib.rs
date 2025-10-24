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

    pub extra_message: Option<String>,
}

impl Default for Error {
    fn default() -> Error {
        Error {
            // please don't use this value
            kind: ErrorKind::InvalidUtf8,
            spans: vec![],
            extra_message: None,
        }
    }
}
