use crate::def::Def;
use crate::error::{MirError, MirErrorKind};
use crate::warn::{MirWarning, MirWarningKind};
use sodigy_error::UniversalError;
use sodigy_high_ir::HirSession;
use sodigy_intern::InternSession;
use sodigy_session::{
    SessionDependency,
    SessionOutput,
    SessionSnapshot,
    SodigySession,
};
use sodigy_uid::Uid;
use std::collections::HashMap;

mod endec;

pub struct MirSession {
    errors: Vec<MirError>,
    warnings: Vec<MirWarning>,
    func_defs: HashMap<Uid, Def>,
    interner: InternSession,
    snapshots: Vec<SessionSnapshot>,
    dependencies: Vec<SessionDependency>,
    previous_errors: Vec<UniversalError>,
}

impl MirSession {
    pub fn from_hir_session(session: &HirSession) -> Self {
        MirSession {
            errors: vec![],
            warnings: vec![],
            func_defs: HashMap::new(),
            interner: session.get_interner_cloned(),
            snapshots: vec![],
            dependencies: session.get_dependencies().clone(),
            previous_errors: session.get_all_errors_and_warnings(),
        }
    }

    pub fn merge_hir(&mut self, hir: &HirSession) -> Result<(), ()> {
        // TODO
        // 1. iterate all the `FuncDef`s in hir,
        // 2. lower the `FuncDef`s to Mir `Def`.

        Ok(())
    }
}

impl SodigySession<MirError, MirErrorKind, MirWarning, MirWarningKind, HashMap<Uid, Def>, Def> for MirSession {
    fn get_errors(&self) -> &Vec<MirError> {
        &self.errors
    }

    fn get_errors_mut(&mut self) -> &mut Vec<MirError> {
        &mut self.errors
    }

    fn get_warnings(&self) -> &Vec<MirWarning> {
        &self.warnings
    }

    fn get_warnings_mut(&mut self) -> &mut Vec<MirWarning> {
        &mut self.warnings
    }

    fn get_previous_errors(&self) -> &Vec<UniversalError> {
        &self.previous_errors
    }

    fn get_previous_errors_mut(&mut self) -> &mut Vec<UniversalError> {
        &mut self.previous_errors
    }

    fn get_results(&self) -> &HashMap<Uid, Def> {
        &self.func_defs
    }

    fn get_results_mut(&mut self) -> &mut HashMap<Uid, Def> {
        &mut self.func_defs
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

// don't use this. just use session.get_results_mut().insert()
impl SessionOutput<Def> for HashMap<Uid, Def> {
    fn pop(&mut self) -> Option<Def> {
        unreachable!()
    }

    fn push(&mut self, v: Def) {
        unreachable!()
    }

    fn clear(&mut self) {
        self.clear();
    }

    fn len(&self) -> usize {
        self.len()
    }
}
