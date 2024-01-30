use crate::error::AstError;
use crate::stmt::Stmt;
use crate::warn::AstWarning;
use sodigy_intern::InternSession;
use sodigy_parse::ParseSession;
use sodigy_session::{SessionDependency, SessionSnapshot, SodigySession};

pub struct AstSession {
    errors: Vec<AstError>,
    warnings: Vec<AstWarning>,
    stmts: Vec<Stmt>,
    interner: InternSession,
    snapshots: Vec<SessionSnapshot>,
    dependencies: Vec<SessionDependency>,
}

impl AstSession {
    pub fn from_parse_session(session: &ParseSession) -> Self {
        AstSession {
            errors: vec![],
            warnings: vec![],
            stmts: vec![],
            interner: session.get_interner_cloned(),
            snapshots: vec![],
            dependencies: session.get_dependencies().clone(),
        }
    }
}

impl SodigySession<AstError, AstWarning, Vec<Stmt>, Stmt> for AstSession {
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

    fn get_dependencies(&self) -> &Vec<SessionDependency> {
        &self.dependencies
    }

    fn get_dependencies_mut(&mut self) -> &mut Vec<SessionDependency> {
        &mut self.dependencies
    }
}
