use colored::Colorize;
use sodigy_endec::EndecError;
use sodigy_files::FileError;
use sodigy_span::SpanRange;

/// Any error type that implements SodigyError can be converted to this type.
/// The compiler uses this type to manage all the errors and warnings.
pub struct UniversalError {
    pub(crate) rendered: String,

    /// It's used to sort the errors by span.
    pub(crate) first_span: SpanRange,

    /// It's used to remove duplicate errors.
    pub(crate) hash: u64,
}

impl UniversalError {
    pub fn rendered(&self) -> &String {
        &self.rendered
    }

    pub fn first_span(&self) -> SpanRange {
        self.first_span
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }
}

impl From<FileError> for UniversalError {
    fn from(e: FileError) -> UniversalError {
        UniversalError {
            rendered: format!(
                "{}\n{}",
                "[Error while doing File IO]".red(),
                e.render_error(),
            ),
            first_span: SpanRange::dummy(0x608e7df7),
            hash: e.hash_u64(),
        }
    }
}

impl From<EndecError> for UniversalError {
    fn from(e: EndecError) -> UniversalError {
        UniversalError {
            rendered: format!(
                "{}\n{}",
                "[Error while decoding a file]".red(),
                e.render_error(),
            ),
            first_span: SpanRange::dummy(0x20060f7a),
            hash: e.hash_u64(),
        }
    }
}
