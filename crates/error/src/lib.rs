use sodigy_span::{RenderableSpan, Span};

mod endec;
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

impl Error {
    pub fn todo(id: u32, message: &str, span: Span) -> Error {
        Error {
            kind: ErrorKind::Todo { id, message: message.to_string() },
            spans: span.simple_error(),
            note: None,
        }
    }
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

// I defined it here because it's usually for error messages.
// Please make sure that `strs.len() > 0`
pub fn comma_list_strs(
    strs: &[String],
    open_quote: &str,
    close_quote: &str,
    and_or: &str,
) -> String {
    match strs.len() {
        0 => String::from("Internal Compiler Error"),
        1 => format!("{open_quote}{}{close_quote}", strs[0]),
        2 => format!("{open_quote}{}{close_quote} {and_or} {open_quote}{}{close_quote}", strs[0], strs[1]),
        3.. => format!("{open_quote}{}{close_quote}, {}", strs[0], comma_list_strs(&strs[1..], open_quote, close_quote, and_or)),
    }
}
