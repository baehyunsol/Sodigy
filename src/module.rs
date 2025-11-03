use crate::Error;
use sodigy_error::{Error as SodigyError, ErrorKind as SodigyErrorKind};
use sodigy_fs_api::exists;
use sodigy_session::{DummySession, Session};
use sodigy_span::Span;
use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ModulePath {
    path: Vec<String>,
}

impl ModulePath {
    pub fn lib() -> ModulePath {
        ModulePath { path: vec![] }
    }

    pub fn is_lib(&self) -> bool {
        self.path.is_empty()
    }

    #[must_use = "method returns a new value and does not mutate the original value"]
    pub fn join(&self, module: String) -> ModulePath {
        let mut path = self.path.clone();
        path.push(module);
        ModulePath { path }
    }

    pub fn get_file_path(
        &self,

        // for error message
        span: Span,
        intermediate_dir: &str,
        error_note: Option<String>,
    ) -> Result<String, Error> {
        let result = if self.is_lib() {
            // TODO: how about `src/lib/mod.sdg`?
            if exists("src/lib.sdg") {
                Ok(String::from("src/lib.sdg"))
            } else {
                // What kinda error message?
                // We need something other than `ModuleFileNotFound`, because it's not a module.
                todo!()
            }
        }

        else {
            let joined = self.to_string();
            let candidate1 = format!("src/{joined}.sdg");
            let candidate2 = format!("src/{joined}/mod.sdg");

            match (exists(&candidate1), exists(&candidate2)) {
                (true, true) => Err(SodigyErrorKind::MultipleModuleFiles { module: self.to_string() }),
                (false, false) => Err(SodigyErrorKind::ModuleFileNotFound { module: self.to_string() }),
                (true, false) => Ok(candidate1),
                (false, true) => Ok(candidate2),
            }
        };

        match result {
            Ok(path) => Ok(path),
            Err(e) => {
                let dummy_session = DummySession {
                    errors: vec![SodigyError {
                        kind: e,
                        spans: span.simple_error(),
                        note: error_note,
                    }],
                    warnings: vec![],
                    intermediate_dir: intermediate_dir.to_string(),
                };
                dummy_session.continue_or_dump_errors().map_err(|_| Error::CompileError)?;
                unreachable!()
            },
        }
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.path.join("/"))
    }
}
