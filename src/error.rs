use sodigy_endec::DecodeError;
use sodigy_error::Error as SodigyError;
use sodigy_fs_api::FileError;
use sodigy_session::{DummySession, Session};

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

// Sometime you have a `sodigy_error::Error`, but no session
// to dump it. This trait creates a dummy session, dumps the error
// and returns `Error::CompileError`
pub trait QuickError<T> {
    fn continue_or_dump_error(self, intermediate_dir: &str) -> Result<T, Error>;
}

impl<T> QuickError<T> for Result<T, SodigyError> {
    fn continue_or_dump_error(self, intermediate_dir: &str) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let mut session = DummySession {
                    errors: vec![e],
                    warnings: vec![],
                    intermediate_dir: intermediate_dir.to_string(),
                };
                session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;
                unreachable!()
            },
        }
    }
}
