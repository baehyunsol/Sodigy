use crate::error::{LexError, LexErrorKind};
use crate::token::{Token, TokenKind};
use crate::warn::{LexWarning, LexWarningKind};

use sodigy_config::CompilerOption;
use sodigy_error::UniversalError;
use sodigy_intern::InternSession;
use sodigy_session::{
    SessionSnapshot,
    SodigySession,
};
use sodigy_span::SpanRange;

#[derive(Clone)]
pub struct LexSession {
    tokens: Vec<Token>,
    errors: Vec<LexError>,
    warnings: Vec<LexWarning>,
    interner: InternSession,
    snapshots: Vec<SessionSnapshot>,
    compiler_option: CompilerOption,

    // must be empty because it doesn't have a previous session
    previous_errors: Vec<UniversalError>,
    previous_warnings: Vec<UniversalError>,
}

impl LexSession {
    pub fn new(compiler_option: CompilerOption) -> Self {
        LexSession {
            tokens: vec![],
            errors: vec![],
            warnings: vec![],
            interner: InternSession::new(),
            snapshots: vec![],
            compiler_option,
            previous_errors: vec![],
            previous_warnings: vec![],
        }
    }

    pub fn flush_tokens(&mut self) {
        self.tokens.clear();
    }

    /// EXPENSIVE
    pub fn dump_tokens(&self) -> String {
        self.tokens.iter().map(|t| t.to_string()).collect::<Vec<String>>().concat()
    }

    pub fn try_push_whitespace(&mut self, span: SpanRange) {
        match self.tokens.last() {
            Some(t) if t.is_whitespace() => {
                // nop
            },
            _ => {
                self.tokens.push(Token {
                    kind: TokenKind::Whitespace,
                    span,
                });
            },
        }
    }
}

impl SodigySession<LexError, LexErrorKind, LexWarning, LexWarningKind, Vec<Token>, Token> for LexSession {
    fn get_errors(&self) -> &Vec<LexError> {
        &self.errors
    }

    fn get_errors_mut(&mut self) -> &mut Vec<LexError> {
        &mut self.errors
    }

    fn get_warnings(&self) -> &Vec<LexWarning> {
        &self.warnings
    }

    fn get_warnings_mut(&mut self) -> &mut Vec<LexWarning> {
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

    fn get_results(&self) -> &Vec<Token> {
        &self.tokens
    }

    fn get_results_mut(&mut self) -> &mut Vec<Token> {
        &mut self.tokens
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
