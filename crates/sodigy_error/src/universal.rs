use crate::{ErrorContext, RenderError, render_error_title};
use sodigy_endec::EndecError;
use sodigy_files::FileError;
use sodigy_span::{ColorScheme, SpanRange, render_spans};
use std::collections::hash_map;
use std::hash::Hasher;

/// Any error type that implements SodigyError can be converted to this type.
/// The compiler uses this type to manage all the errors and warnings.
pub struct UniversalError {
    pub(crate) context: String,

    // `message` includes rendered spans
    pub(crate) message: String,
    pub is_warning: bool,

    /// It's used to sort the errors by span.
    pub(crate) first_span: SpanRange,

    /// It's used to remove duplicate errors.
    pub(crate) hash: u64,
}

impl UniversalError {
    pub fn new(
        context: ErrorContext,
        span: Option<SpanRange>,
        is_warning: bool,

        // those of SodigyErrorKind
        msg: String,
        help: String,
    ) -> Self {
        let message = format!(
            "{msg}{}{}",
            if help.is_empty() {
                String::new()
            } else {
                format!("\n{help}")
            },
            if let Some(span) = span {
                format!(
                    "\n{}",
                    render_spans(
                        &[span],
                        if is_warning {
                            ColorScheme::warning()
                        } else {
                            ColorScheme::error()
                        },
                    ),
                )
            } else {
                String::new()
            },
        );

        let mut hasher = hash_map::DefaultHasher::new();
        hasher.write(message.as_bytes());

        UniversalError {
            hash: hasher.finish(),
            message,
            context: context.render_error(),
            is_warning,
            first_span: span.unwrap_or_else(|| SpanRange::dummy(0x81f82572)),
        }
    }

    pub fn rendered(&self) -> String {
        let title = render_error_title(
            self.context.clone(),
            self.is_warning,
        );

        format!("{title}\n{}", self.message)
    }

    pub fn first_span(&self) -> SpanRange {
        self.first_span
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn append_message(&mut self, m: &str) {
        self.message = format!("{}\n{m}", self.message);
    }
}

// TODO: `From<FileError> for UniversalError` and `From<EndecError> for UniversalError` look the same
impl From<FileError> for UniversalError {
    fn from(e: FileError) -> UniversalError {
        UniversalError {
            context: e.context.render_error(),
            message: e.render_error(),
            is_warning: false,
            first_span: SpanRange::dummy(0x608e7df7),
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
            first_span: SpanRange::dummy(0x20060f7a),
            hash: e.hash_u64(),
        }
    }
}
