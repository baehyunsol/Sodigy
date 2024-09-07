use crate::CompilerOption;
use crate::error::{ClapError, ClapErrorKind};
use crate::warn::{ClapWarning, ClapWarningKind};
use sodigy_error::UniversalError;
use sodigy_intern::InternSession;
use sodigy_session::{
    SessionSnapshot,
    SodigySession,
};

pub struct ClapSession {
    pub(crate) errors: Vec<ClapError>,
    pub(crate) warnings: Vec<ClapWarning>,
    pub(crate) result: CompilerOption,
    pub(crate) interner: InternSession,
    pub(crate) snapshots: Vec<SessionSnapshot>,

    // it's used for the return value of `previous_errors` and `previous_warnings`
    empty_vector: Vec<UniversalError>,
}

impl ClapSession {
    pub fn with_result(result: CompilerOption) -> Self {
        ClapSession {
            result,
            ..ClapSession::default()
        }
    }

    pub fn with_errors(errors: Vec<ClapError>) -> Self {
        ClapSession {
            errors,
            ..ClapSession::default()
        }
    }
}

impl Default for ClapSession {
    fn default() -> Self {
        ClapSession {
            errors: vec![],
            warnings: vec![],
            result: CompilerOption::default(),
            interner: InternSession::new(),
            snapshots: vec![],
            empty_vector: vec![],
        }
    }
}

impl SodigySession<ClapError, ClapErrorKind, ClapWarning, ClapWarningKind, CompilerOption, CompilerOption> for ClapSession {
    fn get_errors(&self) -> &Vec<ClapError> {
        &self.errors
    }

    fn get_errors_mut(&mut self) -> &mut Vec<ClapError> {
        &mut self.errors
    }

    fn get_warnings(&self) -> &Vec<ClapWarning> {
        &self.warnings
    }

    fn get_warnings_mut(&mut self) -> &mut Vec<ClapWarning> {
        &mut self.warnings
    }

    fn get_previous_errors(&self) -> &Vec<UniversalError> {
        &self.empty_vector
    }

    fn get_previous_errors_mut(&mut self) -> &mut Vec<UniversalError> {
        &mut self.empty_vector
    }

    fn get_previous_warnings(&self) -> &Vec<UniversalError> {
        &self.empty_vector
    }

    fn get_previous_warnings_mut(&mut self) -> &mut Vec<UniversalError> {
        &mut self.empty_vector
    }

    fn get_results(&self) -> &CompilerOption {
        &self.result
    }

    fn get_results_mut(&mut self) -> &mut CompilerOption {
        &mut self.result
    }

    fn get_interner(&mut self) -> &mut InternSession {
        &mut self.interner
    }

    fn get_interner_cloned(&self) -> InternSession {
        self.interner.clone()
    }

    fn get_snapshots_mut(&mut self) -> &mut Vec<SessionSnapshot> {
        &mut self.snapshots
    }

    // `CompilerOption` is not initialized yet
    fn get_compiler_option(&self) -> &CompilerOption {
        unreachable!()
    }
}
