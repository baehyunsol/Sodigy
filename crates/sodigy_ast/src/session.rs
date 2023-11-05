use crate::err::AstError;
use crate::stmt::Stmt;
use crate::warn::AstWarning;
use sodigy_intern::{InternedString, InternSession};
use sodigy_parse::ParseSession;

pub struct AstSession {
    errors: Vec<AstError>,
    warnings: Vec<AstWarning>,
    stmts: Vec<Stmt>,
    interner: InternSession,
}

impl AstSession {
    pub fn from_parse_session(session: &ParseSession) -> Self {
        AstSession {
            errors: vec![],
            warnings: vec![],
            stmts: vec![],
            interner: session.interner.clone(),
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

    pub fn err_if_has_err(&self) -> Result<(), ()> {
        if self.errors.is_empty() {
            Ok(())
        }

        else {
            Err(())
        }
    }
}
