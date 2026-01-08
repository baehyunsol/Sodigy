use crate::CompileStage;
use sodigy_endec::DecodeError;
use sodigy_fs_api::FileError;

/// It decides the exit code of the compiler process.
pub enum Error {
    /// When the interpreter panics (not Rust's panic, but Sodigy's panic).
    RuntimeError,

    /// Error in Sodigy code (directly converted from `sodigy_error::Error`).
    /// Some FileError can be converted to CompileError, if the error has something
    /// to do with Sodigy.
    CompileError,

    FileError(FileError),
    DecodeError(DecodeError),
    CliError(ragit_cli::Error),
    MpscError,
    IrCacheNotFound(CompileStage),

    /// Errors other than the above errors.
    MiscError,
}

impl Error {
    // NOTE: rust's `panic!` macro always uses exit code 101.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::RuntimeError => 10,
            Error::CompileError => 11,
            Error::CliError(_) => 12,

            // `RuntimeError`, `CompileError` and `CliError` are obvious, but
            // the other variants are subject to change.
            _ => 13,
        }
    }
}

impl From<FileError> for Error {
    fn from(e: FileError) -> Error {
        Error::FileError(e)
    }
}

impl From<DecodeError> for Error {
    fn from(e: DecodeError) -> Error {
        Error::DecodeError(e)
    }
}

impl From<ragit_cli::Error> for Error {
    fn from(e: ragit_cli::Error) -> Error {
        Error::CliError(e)
    }
}

impl<T> From<std::sync::mpsc::SendError<T>> for Error {
    fn from(_: std::sync::mpsc::SendError<T>) -> Error {
        Error::MpscError
    }
}
