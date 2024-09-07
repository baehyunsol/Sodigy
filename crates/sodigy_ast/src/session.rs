use crate::error::{AstError, AstErrorKind};
use crate::stmt::Stmt;
use crate::warn::{AstWarning, AstWarningKind};
use sodigy_config::CompilerOption;
use sodigy_error::UniversalError;
use sodigy_intern::InternSession;
use sodigy_parse::ParseSession;
use sodigy_session::{
    SessionSnapshot,
    SodigySession,
};

pub struct AstSession {
    errors: Vec<AstError>,
    warnings: Vec<AstWarning>,
    stmts: Vec<Stmt>,
    interner: InternSession,
    snapshots: Vec<SessionSnapshot>,
    compiler_option: CompilerOption,
    previous_errors: Vec<UniversalError>,
    previous_warnings: Vec<UniversalError>,
}

impl AstSession {
    pub fn from_parse_session(session: &ParseSession) -> Self {
        AstSession {
            errors: vec![],
            warnings: vec![],
            stmts: vec![],
            interner: session.get_interner_cloned(),
            snapshots: vec![],
            compiler_option: session.get_compiler_option().clone(),
            previous_errors: session.get_all_errors(),
            previous_warnings: session.get_all_warnings(),
        }
    }

    pub fn get_or_pattern_expansion_limit(&self) -> usize {
        self.compiler_option.or_pattern_expansion_limit
    }
}

impl SodigySession<AstError, AstErrorKind, AstWarning, AstWarningKind, Vec<Stmt>, Stmt> for AstSession {
    fn get_errors(&self) -> &Vec<AstError> {
        &self.errors
    }

    fn get_errors_mut(&mut self) -> &mut Vec<AstError> {
        &mut self.errors
    }

    fn get_warnings(&self) -> &Vec<AstWarning> {
        &self.warnings
    }

    fn get_warnings_mut(&mut self) -> &mut Vec<AstWarning> {
        &mut self.warnings
    }

    fn get_previous_errors(&self) -> &Vec<UniversalError> {
        &self.previous_errors
    }

    fn get_previous_errors_mut(&mut self) -> &mut Vec<UniversalError> {
        &mut self.previous_errors
    }

    fn get_previous_warnings(&self) -> &Vec<UniversalError> {
        &self.previous_warnings
    }

    fn get_previous_warnings_mut(&mut self) -> &mut Vec<UniversalError> {
        &mut self.previous_warnings
    }

    fn get_results(&self) -> &Vec<Stmt> {
        &self.stmts
    }

    fn get_results_mut(&mut self) -> &mut Vec<Stmt> {
        &mut self.stmts
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

    fn get_compiler_option(&self) -> &CompilerOption {
        &self.compiler_option
    }
}
