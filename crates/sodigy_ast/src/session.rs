use crate::err::AstError;
use crate::stmt::Stmt;
use crate::warn::AstWarning;
use sodigy_intern::{InternedString, InternSession};
use sodigy_parse::ParseSession;

pub struct AstSession {
    pub errors: Vec<AstError>,
    pub warnings: Vec<AstWarning>,
    stmts: Vec<Stmt>,
    interner: InternSession,
    snapshots: Vec<AstSessionSnapshot>,
}

impl AstSession {
    pub fn from_parse_session(session: &ParseSession) -> Self {
        AstSession {
            errors: vec![],
            warnings: vec![],
            stmts: vec![],
            interner: session.interner.clone(),
            snapshots: vec![],
        }
    }

    pub fn push_stmt(&mut self, stmt: Stmt) {
        self.stmts.push(stmt);
    }

    pub fn get_stmts(&self) -> &Vec<Stmt> {
        &self.stmts
    }

    pub fn intern_string(&mut self, string: Vec<u8>) -> InternedString {
        self.interner.intern_string(string)
    }

    pub fn unintern_string_fast(&mut self, string: InternedString) -> Option<&[u8]> {
        self.interner.unintern_string_fast(string)
    }

    pub fn unintern_string(&mut self, string: InternedString) -> Option<&[u8]> {
        self.interner.unintern_string(string)
    }

    pub fn push_error(&mut self, error: AstError) {
        self.errors.push(error);
    }

    pub fn pop_error(&mut self) -> Option<AstError> {
        self.errors.pop()
    }

    pub fn get_errors(&self) -> &Vec<AstError> {
        &self.errors
    }

    pub fn push_warning(&mut self, warning: AstWarning) {
        self.warnings.push(warning);
    }

    pub fn get_warnings(&self) -> &Vec<AstWarning> {
        &self.warnings
    }

    // TODO: no more `err_if_has_err`
    pub fn err_if_has_err(&self) -> Result<(), ()> {
        if self.errors.is_empty() {
            Ok(())
        }

        else {
            Err(())
        }
    }

    pub fn take_snapshot(&mut self) {
        self.snapshots.push(AstSessionSnapshot {
            errors: self.errors.len(),
            warnings: self.warnings.len(),
            stmts: self.stmts.len(),
        });
    }

    // there's no point in returning the snapshot. It only tells the caller whether
    // self.snapshots is empty or not
    pub fn pop_snapshot(&mut self) -> Result<(), ()> {
        self.snapshots.pop().map(|_| ()).ok_or(())
    }

    pub fn restore_to_last_snapshot(&mut self) {
        let last_snapshot = self.snapshots.pop().unwrap();

        while self.errors.len() > last_snapshot.errors {
            self.errors.pop().unwrap();
        }

        while self.warnings.len() > last_snapshot.warnings {
            self.warnings.pop().unwrap();
        }

        while self.stmts.len() > last_snapshot.stmts {
            self.stmts.pop().unwrap();
        }
    }
}

// for optimization, it only stores the lengths. It's okay for now because
// snapshots are rarely used. If it causes problems, we should copy the entire
// vectors
struct AstSessionSnapshot {
    pub errors: usize,
    pub warnings: usize,
    pub stmts: usize,
}
