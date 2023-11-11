use crate::{ParseError, TokenTree};
use crate::warn::ParseWarning;
use sodigy_intern::{InternedNumeric, InternedString, InternSession};
use sodigy_lex::LexSession;
use sodigy_number::SodigyNumber;

pub struct ParseSession {
    pub tokens: Vec<TokenTree>,
    pub errors: Vec<ParseError>,
    pub warnings: Vec<ParseWarning>,
    pub interner: InternSession,
}

impl ParseSession {
    pub fn from_lex_session(s: &LexSession) -> Self {
        ParseSession {
            tokens: vec![],
            errors: vec![],
            warnings: vec![],
            interner: s.get_interner().clone(),
        }
    }

    pub fn push_token(&mut self, token: TokenTree) {
        self.tokens.push(token);
    }

    pub fn push_error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    pub fn push_warning(&mut self, warning: ParseWarning) {
        self.warnings.push(warning);
    }

    pub fn intern_string(&mut self, string: Vec<u8>) -> InternedString {
        self.interner.intern_string(string)
    }

    pub fn intern_numeric(&mut self, numeric: SodigyNumber) -> InternedNumeric {
        self.interner.intern_numeric(numeric)
    }

    pub fn get_tokens(&self) -> &Vec<TokenTree> {
        &self.tokens
    }

    pub fn flush_tokens(&mut self) {
        self.tokens.clear();
    }

    /// EXPENSIVE
    pub fn dump_tokens(&self) -> String {
        self.tokens.iter().map(|t| format!("{t}")).collect::<Vec<String>>().join(" ")
    }

    pub fn get_errors(&self) -> &Vec<ParseError> {
        &self.errors
    }

    pub fn get_warnings(&self) -> &Vec<ParseWarning> {
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
