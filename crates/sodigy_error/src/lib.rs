#![deny(unused_imports)]

use colored::Colorize;
use sodigy_files::global_file_session;
use sodigy_intern::InternSession;
use sodigy_span::{ColorScheme, SpanRange, render_spans};
use std::collections::{HashSet, hash_map};
use std::hash::Hasher;

mod dist;
mod expected_token;
mod fmt;
mod universal;

pub use dist::substr_edit_distance;
pub use expected_token::ExpectedToken;
pub use fmt::RenderError;
pub use universal::UniversalError;

#[derive(Clone, Debug)]
pub struct ExtraErrInfo {
    // very context-specific message for an error,
    // for example, there may be a very specific context for `UnexpectedToken`s (suspicious typos, deprecated features, etc...)
    msg: String,
    context: ErrorContext,
    show_span: bool,
}

impl ExtraErrInfo {
    pub fn none() -> Self {
        ExtraErrInfo {
            msg: String::new(),
            context: ErrorContext::Unknown,
            show_span: true,
        }
    }

    pub fn at_context(context: ErrorContext) -> Self {
        ExtraErrInfo {
            msg: String::new(),
            context,
            show_span: true,
        }
    }

    pub fn has_message(&self) -> bool {
        !self.msg.is_empty()
    }

    pub fn set_err_context(&mut self, context: ErrorContext) -> &mut Self {
        self.context = context;

        self
    }

    pub fn set_message(&mut self, msg: String) -> &mut Self {
        self.msg = msg;

        self
    }

    pub fn set_show_span(&mut self, show_span: bool) -> &mut Self {
        self.show_span = show_span;

        self
    }
}

// TODO: Option<SpanRange> for ErrorContext
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorContext {
    Unknown,
    ParsingCommandLine,
    ExpandingMacro,
    Lexing,
    LexingNumericLiteral,
    ParsingLetStatement,
    ParsingImportStatement,
    ParsingFuncName,
    ParsingFuncRetType,
    ParsingFuncBody,
    ParsingFuncArgs,
    ParsingEnumBody,
    ParsingStructBody,
    ParsingStructInit,
    ParsingMatchBody,
    ParsingBranchCondition,
    ParsingLambdaBody,
    ParsingScopeBlock,
    ParsingFormattedString,
    ParsingPattern,
    ParsingTypeInPattern,
}

pub trait SodigyError<K: SodigyErrorKind> {
    fn get_mut_error_info(&mut self) -> &mut ExtraErrInfo;

    fn get_error_info(&self) -> &ExtraErrInfo;

    // errors must have at least 1 span
    fn get_first_span(&self) -> SpanRange;

    fn get_spans(&self) -> &[SpanRange];

    fn err_kind(&self) -> &K;

    /// Errors at different passes have different indices.
    /// For example, lex error, parse error and ast error have different ones.
    fn index(&self) -> u32;

    // override this when it's a warning
    fn is_warning(&self) -> bool {
        false
    }

    fn color_scheme(&self) -> ColorScheme {
        if self.is_warning() {
            ColorScheme::warning()
        }

        else {
            ColorScheme::error()
        }
    }

    fn set_err_context(&mut self, context: ErrorContext) -> &mut Self {
        self.get_mut_error_info().set_err_context(context);

        self
    }

    // sets the error context when,\
    // 1. it's not set previously
    // 2. the given context is not none
    fn try_set_err_context(&mut self, context: Option<ErrorContext>) -> &mut Self {
        let ctx = self.get_mut_error_info();

        if ctx.context == ErrorContext::Unknown {
            if let Some(err_ctx) = context {
                ctx.context = err_ctx;
            }
        }

        self
    }

    fn set_message(&mut self, message: String) -> &mut Self {
        self.get_mut_error_info().set_message(message);

        self
    }

    fn to_universal(&self) -> UniversalError {
        let rendered = self.render_error();
        let hash = {
            let mut hasher = hash_map::DefaultHasher::new();
            hasher.write(&self.get_first_span().hash128().to_be_bytes());
            hasher.write(&[self.is_warning() as u8]);
            hasher.write(&self.err_kind().index().to_be_bytes());
            hasher.write(&self.index().to_be_bytes());

            hasher.finish()
        };

        UniversalError {
            rendered,
            first_span: self.get_first_span(),
            hash,
        }
    }

    // This function is VERY VERY EXPENSIVE.
    fn render_error(&self) -> String {
        let mut intern_session = InternSession::new();
        let context = match &self.get_error_info().context {
            ErrorContext::Unknown => String::new(),
            c => format!(" while {c}"),
        };

        let kind = self.err_kind();

        let msg = kind.msg(&mut intern_session);
        let help = match kind.help(&mut intern_session) {
            s if s.is_empty() => String::new(),
            s => format!("\nHelp: {s}"),
        };
        let extra_msg = match &self.get_error_info().msg {
            s if s.is_empty() => String::new(),
            s => format!("\nNote: {s}"),
        };
        let spans = self.get_spans().iter().filter(
            |s| !s.is_dummy()
        ).map(
            |s| *s
        ).collect::<Vec<SpanRange>>();

        let color_scheme = self.color_scheme();

        let span = match &self.get_error_info().show_span {
            true if spans.is_empty() => format!("<NO SPANS AVAILABLE>"),
            true => render_spans(&spans, color_scheme),
            false if spans.is_empty() => String::new(),
            false => show_file_names(&spans),
        };

        let title = if self.is_warning() {
            format!("[Warning]").yellow()
        } else {
            format!("[Error{context}]").red()
        };

        format!(
            "{title}\n{msg}{help}{extra_msg}\n{span}",
        )
    }
}

pub trait SodigyErrorKind {
    // main explanation of this error
    // no capital letters, no dot
    fn msg(&self, _: &mut InternSession) -> String;

    // extra sentences that explain the error
    // if the help msg is empty, it's ignored
    fn help(&self, _: &mut InternSession) -> String;

    /// identifier of this errkind
    fn index(&self) -> u32;
}

pub fn concat_commas(list: &[String], term: &str, prefix: &str, suffix: &str) -> String {
    match list.len() {
        0 => unreachable!(),
        1 => format!("{prefix}{}{suffix}", list[0]),
        2 => format!("{prefix}{}{suffix} {term} {prefix}{}{suffix}", list[0], list[1]),
        _ => format!("{prefix}{}{suffix}, {}", list[0], concat_commas(&list[1..], term, prefix, suffix)),
    }
}

fn show_file_names(spans: &[SpanRange]) -> String {
    let file_session = unsafe { global_file_session() };
    let file_names = spans.iter().map(
        |sp| file_session.render_file_hash(sp.file)
    ).collect::<HashSet<String>>().into_iter().collect::<Vec<String>>();

    concat_commas(&file_names, "and", "<", ">")
}
