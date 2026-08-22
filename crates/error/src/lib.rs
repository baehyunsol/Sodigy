use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::HashSet;

mod dump;
mod endec;
mod kind;
mod lint;
mod token;
mod warning;

#[cfg(test)]
mod tests;

pub use dump::{DumpErrorOption, dump_errors};
pub use kind::{EnumFieldKind, ErrorKind, NameCollisionKind, NotXBut};
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

    pub fn ice(id: u32, span: Span) -> Error {
        Error {
            kind: ErrorKind::InternalCompilerError { id },
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

    // TODO: `FuncParam` is not an item, but `get_attribute_rule` needs this variant.
    //       Maybe we have to rename `ItemKind` to `AttributeKind` or define another enum
    //       for `get_attribute_rule`.
    FuncParam,

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
            ItemKind::FuncParam => "function parameter",
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

/// Sometimes there's an error with a type of a function.
/// The error might have to do with a parameter, or with the return type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ParamIndex {
    Param(usize),
    Return,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TypeVarInfo {
    Ident(InternedString),
    ListExpr,
    ListPattern,
}

// TODO: I'm not sure whether this is the best place to define this enum.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FuncEffect {
    Fn,
    Proc,
    NdetFn,
    NdetProc,
    Callable,

    // It's for effect-inference.
    // NOTE: `Span` is too big, so I'm using `Box<Span>`.
    Var(Box<Span>),
}

impl FuncEffect {
    pub fn from_ndet_and_proc(is_ndet: bool, is_proc: bool) -> FuncEffect {
        match (is_ndet, is_proc) {
            (true, true) => FuncEffect::NdetProc,
            (true, false) => FuncEffect::NdetFn,
            (false, true) => FuncEffect::Proc,
            (false, false) => FuncEffect::Fn,
        }
    }

    pub fn keyword(&self) -> &'static str {
        match self {
            FuncEffect::Fn => "fn",
            FuncEffect::Proc => "proc",
            FuncEffect::NdetFn => "ndet fn",
            FuncEffect::NdetProc => "ndet proc",
            _ => unreachable!(),
        }
    }

    pub fn to_usize(&self) -> usize {
        match self {
            FuncEffect::Fn       => 0b_000,
            FuncEffect::Proc     => 0b_001,
            FuncEffect::NdetFn   => 0b_010,
            FuncEffect::NdetProc => 0b_011,
            FuncEffect::Callable => 0b_111,
            _ => unreachable!(),
        }
    }
}
