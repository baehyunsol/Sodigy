use sodigy_span::Span;

mod kind;
mod level;
mod render;
mod token;
mod warning;

pub use kind::ErrorKind;
pub use level::ErrorLevel;
pub use render::{RenderSpanOption, render_span};
pub use token::ErrorToken;
pub use warning::{Warning, WarningKind};

#[derive(Clone, Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,

    // Some errors have multiple spans (e.g. name collision)
    pub extra_span: Option<Span>,
    pub extra_message: Option<String>,
}

impl Default for Error {
    fn default() -> Error {
        Error {
            // please don't use this value
            kind: ErrorKind::InvalidUtf8,
            span: Span::None,

            // default is for these fields
            extra_span: None,
            extra_message: None,
        }
    }
}
