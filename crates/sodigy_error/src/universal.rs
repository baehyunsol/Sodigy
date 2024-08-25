use crate::{ErrorContext, RenderError, render_error_title, show_file_names};
use smallvec::{SmallVec, smallvec};
use sodigy_endec::EndecError;
use sodigy_files::FileError;
use sodigy_span::{ColorScheme, SpanRange, render_spans};
use std::collections::hash_map;
use std::hash::Hasher;

mod endec;

/// Any error type that implements SodigyError can be converted to this type.
/// The compiler uses this type to manage all the errors and warnings.
#[derive(Clone)]
pub struct UniversalError {
    pub(crate) context: String,

    // `message` includes rendered spans
    pub(crate) message: String,
    pub is_warning: bool,

    pub(crate) spans: SmallVec<[SpanRange; 1]>,
    pub show_span: bool,

    /// It's used to remove duplicate errors.
    pub(crate) hash: u64,
}

impl UniversalError {
    pub fn new(
        context: ErrorContext,
        is_warning: bool,
        show_span: bool,
        span: Option<SpanRange>,

        // those of SodigyErrorKind
        msg: String,
        help: String,
    ) -> Self {
        let message = format!(
            "{msg}{}",
            if help.is_empty() {
                String::new()
            } else {
                format!("\n{help}")
            },
        );

        let mut hasher = hash_map::DefaultHasher::new();
        hasher.write(message.as_bytes());

        let spans = if let Some(span) = span {
            smallvec![span]
        } else {
            smallvec![]
        };

        UniversalError {
            hash: hasher.finish(),
            message,
            context: context.render_error(),
            is_warning,
            show_span,
            spans,
        }
    }

    pub fn rendered(&self) -> String {
        let title = render_error_title(
            self.context.clone(),
            self.is_warning,
        );

        let color_scheme = if self.is_warning {
            ColorScheme::warning()
        } else {
            ColorScheme::error()
        };

        let span = match self.show_span {
            true if self.spans.is_empty() => format!("<NO SPANS AVAILABLE>"),
            true => render_spans(&self.spans, color_scheme),
            false if self.spans.is_empty() => String::new(),
            false => show_file_names(&self.spans),
        };

        format!("{title}\n{}\n{span}", self.message)
    }

    pub fn first_span(&self) -> Option<SpanRange> {
        self.spans.get(0).map(|span| *span)
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn append_message(&mut self, m: &str) {
        self.message = format!("{}\n{m}", self.message);
    }

    pub fn push_span(&mut self, span: SpanRange) {
        self.spans.push(span);
    }
}

// TODO: `From<FileError> for UniversalError` and `From<EndecError> for UniversalError` look the same
impl From<FileError> for UniversalError {
    fn from(e: FileError) -> UniversalError {
        UniversalError {
            context: e.context.render_error(),
            message: e.render_error(),
            is_warning: false,
            show_span: false,
            spans: smallvec![],
            hash: e.hash_u64(),
        }
    }
}

impl From<EndecError> for UniversalError {
    fn from(e: EndecError) -> UniversalError {
        UniversalError {
            context: e.context.render_error(),
            message: e.render_error(),
            is_warning: false,
            show_span: false,
            spans: smallvec![],
            hash: e.hash_u64(),
        }
    }
}
