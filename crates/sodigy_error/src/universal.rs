use colored::Colorize;
use sodigy_endec::EndecError;
use sodigy_files::FileError;
use sodigy_span::SpanRange;
use std::collections::hash_map;
use std::hash::Hasher;

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
            first_span: SpanRange::dummy(9),
            hash: {
                let mut hasher = hash_map::DefaultHasher::new();
                hasher.write(&e.kind.hash_u64().to_be_bytes());

                if let Some(p) = &e.given_path {
                    hasher.write(p.as_bytes());
                }

                hasher.finish()
            },
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
            first_span: SpanRange::dummy(10),
            hash: todo!(),
        }
    }
}
