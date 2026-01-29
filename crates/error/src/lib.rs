use sodigy_span::{RenderableSpan, Span};
use std::collections::HashSet;

mod dump;
mod endec;
mod kind;
mod lint;
mod token;
mod warning;

pub use dump::{DumpErrorOption, dump_errors};
pub use kind::{ErrorKind, NameCollisionKind, NotExprBut, NotStructBut, NotTypeBut};
pub use lint::{Lint, LintKind};
pub use token::ErrorToken;
pub use warning::{Warning, WarningKind};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

/// By default,
///
/// 1. If the compiler finds an `Error`, it halts the compilation almost immediately.
/// 2. If the compiler finds a `Warning`, it continues the compilation, and dumps the warnings.
/// 3. If the compiler finds a `Lint`, it continues the compilation, and doesn't dump the lints.
///
/// But the user can forbid/warn/allow `Warning`s and `Lint`s.
/// If the compiler finds a forbidden `Warning`/`Lint`, it halts the compilation before the optimization stage
/// and dumps the forbidden `Warning`/`Lint` as if it were an error.
/// If the compiler finds a warned `Lint`, it dumps the lint as if it were a warning.
/// If the compiler finds an allowed `Warning`, it just ignores the `Warning` as if it were a `Lint`.
#[derive(Clone, Copy, Debug)]
pub enum ErrorLevel {
    Error,
    Warning,
    Lint,
}

#[derive(Clone, Copy, Debug)]
pub enum CustomErrorLevel {
    Forbid,
    Warn,
    Allow,
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ItemKind {
    Alias,
    Assert,
    Enum,
    EnumVariant,
    Func,
    Let,
    Module,
    Struct,
    Use,
}

impl ItemKind {
    pub fn render(&self) -> &'static str {
        match self {
            ItemKind::Alias => "type alias",
            ItemKind::Assert => "assertion",
            ItemKind::Enum => "enum",
            ItemKind::EnumVariant => "enum variant",
            ItemKind::Func => "function",
            ItemKind::Let => "`let` statement",
            ItemKind::Module => "module",
            ItemKind::Struct => "struct",
            ItemKind::Use => "`use` statement",
        }
    }
}

pub fn deduplicate(errors: &mut Vec<Error>) -> Vec<Error> {
    errors.drain(..).collect::<HashSet<_>>().into_iter().collect()
}
