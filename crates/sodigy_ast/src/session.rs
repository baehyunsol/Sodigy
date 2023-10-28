use crate::err::AstError;
use crate::stmt::Stmt;
use sodigy_intern::{InternedString, InternSession};
use sodigy_parse::ParseSession;

pub struct AstSession {
    errors: Vec<AstError>,
    stmts: Vec<Stmt>,
    interner: InternSession,
}

impl AstSession {
    pub fn from_parse_session(session: &ParseSession) -> Self {
        AstSession {
            errors: vec![],
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

    pub fn unintern_string(&mut self, string: InternedString) -> Option<&[u8]> {
        self.interner.unintern_string(string)
    }

    pub fn push_error(&mut self, error: AstError) {
        self.errors.push(error);
    }

    pub fn get_errors(&self) -> &Vec<AstError> {
        &self.errors
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
