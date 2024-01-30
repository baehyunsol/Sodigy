use crate::error::LexError;
use crate::token::{Token, TokenKind};
use crate::warn::LexWarning;

use sodigy_intern::InternSession;
use sodigy_session::{SessionDependency, SessionSnapshot, SodigySession};
use sodigy_span::SpanRange;

#[derive(Clone)]
pub struct LexSession {
    tokens: Vec<Token>,
    errors: Vec<LexError>,
    warnings: Vec<LexWarning>,
    interner: InternSession,
    snapshots: Vec<SessionSnapshot>,
    dependencies: Vec<SessionDependency>,
}

impl LexSession {
    pub fn new() -> Self {
        LexSession {
            tokens: vec![],
            errors: vec![],
            warnings: vec![],
            interner: InternSession::new(),
            snapshots: vec![],
            dependencies: vec![],
        }
    }

    pub fn flush_tokens(&mut self) {
        self.tokens.clear();
    }

    /// EXPENSIVE
    pub fn dump_tokens(&self) -> String {
        self.tokens.iter().map(|t| t.to_string()).collect::<Vec<String>>().concat()
    }

    pub fn try_push_whitespace(&mut self) {
        match self.tokens.last() {
            Some(t) if t.is_whitespace() => {
                // nop
            },
            _ => {
                self.tokens.push(Token {
                    kind: TokenKind::Whitespace,
                    span: SpanRange::dummy(0x114835f6),
                });
            }
        }
    }
}

impl SodigySession<LexError, LexWarning, Vec<Token>, Token> for LexSession {
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

    fn get_dependencies(&self) -> &Vec<SessionDependency> {
        &self.dependencies
    }

    fn get_dependencies_mut(&mut self) -> &mut Vec<SessionDependency> {
        &mut self.dependencies
    }
}
