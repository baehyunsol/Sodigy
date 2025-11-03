use sodigy_endec::DecodeError;
use sodigy_fs_api::FileError;

// Errors are already processed (e.g. compile errors are already dumped to stderr).
// We only wanna know what kind of error it is.
#[derive(Clone, Debug)]
pub enum Error {
    CompileError,
    FileError(FileError),
    DecodeError(DecodeError),
    CliError,
    TestError,
    MpscError,
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

impl<T> From<std::sync::mpsc::SendError<T>> for Error {
    fn from(_: std::sync::mpsc::SendError<T>) -> Error {
        Error::MpscError
    }
}
