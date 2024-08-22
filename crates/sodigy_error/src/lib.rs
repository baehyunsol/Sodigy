#![deny(unused_imports)]

use colored::Colorize;
use sodigy_endec::{
    DumpJson,
    JsonObj,
    json_key_value_table,
};
use sodigy_files::global_file_session;
use sodigy_intern::InternSession;
use sodigy_span::{ColorScheme, SpanRange};
use std::collections::{HashSet, hash_map};
use std::hash::Hasher;

mod ctxt;
mod dist;
mod expected_token;
mod extra_info;
mod fmt;
mod universal;

pub use ctxt::ErrorContext;
pub use dist::substr_edit_distance;
pub use expected_token::ExpectedToken;
pub use extra_info::ExtraErrInfo;
pub use fmt::RenderError;
pub use universal::UniversalError;

pub trait SodigyError<K: SodigyErrorKind> {
    fn get_mut_error_info(&mut self) -> &mut ExtraErrInfo;

    fn get_error_info(&self) -> &ExtraErrInfo;

    fn get_first_span(&self) -> Option<SpanRange>;

    fn get_spans(&self) -> &[SpanRange];

    fn error_kind(&self) -> &K;

    /// Errors at different passes have different indices.
    /// For example, lex error, parse error and ast error have different ones.
    fn index(&self) -> u32;

    /// override this when it's a warning
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

    fn set_error_context(&mut self, context: ErrorContext) -> &mut Self {
        self.get_mut_error_info().set_error_context(context);

        self
    }

    // sets the error context when,\
    // 1. it's not set previously
    // 2. the given context is not none
    fn try_set_error_context(&mut self, context: Option<ErrorContext>) -> &mut Self {
        let ctx = self.get_mut_error_info();

        if ctx.context == ErrorContext::Unknown {
            if let Some(error_ctx) = context {
                ctx.context = error_ctx;
            }
        }

        self
    }

    fn set_message(&mut self, message: String) -> &mut Self {
        self.get_mut_error_info().set_message(message);

        self
    }

    fn to_universal(&self) -> UniversalError {
        let context = self.get_error_info().context.render_error();
        let message = self.render_error();
        let hash = {
            let mut hasher = hash_map::DefaultHasher::new();

            if let Some(span) = self.get_first_span() {
                hasher.write(&span.hash128().to_be_bytes());
            }

            hasher.write(&[self.is_warning() as u8]);
            hasher.write(&self.error_kind().index().to_be_bytes());
            hasher.write(&self.index().to_be_bytes());

            hasher.finish()
        };

        UniversalError {
            context,
            message,
            is_warning: self.is_warning(),
            show_span: self.get_error_info().show_span,
            spans: self.get_spans().into(),
            hash,
        }
    }

    // It only renders `msg`, `help` and `extra_msg`.
    // It doesn't render the title and the spans.
    // In order to see the full error message, you have to 
    // convert this to a UniversalError then call `.rendered()`.
    fn render_error(&self) -> String {
        let mut intern_session = InternSession::new();
        let is_warning = self.is_warning();

        let kind = self.error_kind();

        let msg = format!(
            "{}{:04}: {}",
            if is_warning { "W" } else { "E" },
            self.index() * 100 + self.error_kind().index(),
            kind.msg(&mut intern_session),
        );
        let help = match kind.help(&mut intern_session) {
            s if s.is_empty() => String::new(),
            s => format!("\nHelp: {s}"),
        };
        let extra_msg = match &self.get_error_info().msg {
            s if s.is_empty() => String::new(),
            s => format!("\nNote: {s}"),
        };

        format!(
            "{msg}{help}{extra_msg}",
        )
    }

    // due to Rust's trait coherence rules,
    // it cannot do something like `impl<T: SodigyError> DumpJson for T`
    // we needa boilerplate
    fn dump_json_impl(&self) -> JsonObj {
        let kind = self.error_kind().dump_json_impl();
        let spans = self.get_spans().to_vec().dump_json();
        let extra = self.get_error_info().dump_json();

        json_key_value_table(vec![
            ("kind", kind),
            ("spans", spans),
            ("extra_information", extra),
        ])
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

    // due to Rust's trait coherence rules,
    // it cannot do something like `impl<T: SodigyErrorKind> DumpJson for T`
    // we needa boilerplate
    fn dump_json_impl(&self) -> JsonObj {
        let mut dummy_session = InternSession::new();

        json_key_value_table(vec![
            ("message", self.msg(&mut dummy_session).dump_json()),
            ("help_message", self.help(&mut dummy_session).dump_json()),
        ])
    }
}

pub fn concat_commas(list: &[String], term: &str, prefix: &str, suffix: &str) -> String {
    match list.len() {
        0 => unreachable!(),
        1 => format!("{prefix}{}{suffix}", list[0]),
        2 => format!("{prefix}{}{suffix} {term} {prefix}{}{suffix}", list[0], list[1]),
        _ => format!("{prefix}{}{suffix}, {}", list[0], concat_commas(&list[1..], term, prefix, suffix)),
    }
}

pub fn trim_long_string(s: String, prefix: usize, suffix: usize) -> String {
    let char_len = s.chars().count();

    if char_len > prefix + suffix + 3 {
        format!(
            "{}...{}",
            s.chars().take(prefix).collect::<String>(),
            s.chars().rev().take(suffix).collect::<String>().chars().rev().collect::<String>(),
        )
    }

    else {
        s
    }
}

fn show_file_names(spans: &[SpanRange]) -> String {
    let file_session = unsafe { global_file_session() };
    let file_names = spans.iter().map(
        |sp| file_session.render_file_hash(sp.file)
    ).collect::<HashSet<String>>().into_iter().collect::<Vec<String>>();

    concat_commas(&file_names, "and", "<", ">")
}

pub(crate) fn render_error_title(
    context: String,
    is_warning: bool,
) -> String {
    if is_warning {
        "[Warning]".yellow()
    } else {
        let context = if context.is_empty() {
            String::new()
        } else {
            format!(" while {context}")
        };

        format!("[Error{context}]").red()
    }.to_string()
}
