use crate::CompilerOption;
use crate::error::ClapError;
use crate::warn::ClapWarning;
use sodigy_intern::InternSession;
use sodigy_session::{
    SessionDependency,
    SessionOutput,
    SessionSnapshot,
    SodigySession,
};

pub struct ClapSession {
    pub(crate) errors: Vec<ClapError>,
    pub(crate) warnings: Vec<ClapWarning>,
    pub(crate) result: CompilerOption,
    pub(crate) interner: InternSession,
    pub(crate) snapshots: Vec<SessionSnapshot>,
    pub(crate) dependencies: Vec<SessionDependency>,
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
            dependencies: vec![],
        }
    }
}

impl SodigySession<ClapError, ClapWarning, CompilerOption, CompilerOption> for ClapSession {
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

    fn get_dependencies(&self) -> &Vec<SessionDependency> {
        &self.dependencies
    }

    fn get_dependencies_mut(&mut self) -> &mut Vec<SessionDependency> {
        &mut self.dependencies
    }
}

// don't call these. just use session.get_results_mut()
impl SessionOutput<CompilerOption> for CompilerOption {
    fn pop(&mut self) -> Option<CompilerOption> {
        unreachable!()
    }

    fn push(&mut self, v: CompilerOption) {
        unreachable!()
    }

    fn clear(&mut self) {
        unreachable!()
    }

    fn len(&self) -> usize {
        unreachable!()
    }
}
