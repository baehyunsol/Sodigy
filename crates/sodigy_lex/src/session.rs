use crate::error::LexError;
use crate::token::{Token, TokenKind};

use sodigy_error::{ErrorContext, SodigyError};
use sodigy_intern::{InternSession, InternedString};
use sodigy_span::SpanRange;

#[derive(Clone)]
pub struct LexSession {
    tokens: Vec<Token>,
    errors: Vec<LexError>,
    interner: InternSession,
}

impl LexSession {
    pub fn new() -> Self {
        LexSession {
            tokens: vec![],
            errors: vec![],
            interner: InternSession::new(),
        }
    }

    pub fn intern_string(&mut self, ident: Vec<u8>) -> InternedString {
        self.interner.intern_string(ident)
    }

    pub fn push_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    pub fn push_error(&mut self, mut error: LexError) {
        error.try_set_err_context(Some(ErrorContext::Lexing));
        self.errors.push(error);
    }

    pub fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    pub fn get_interner(&self) -> &InternSession {
        &self.interner
    }

    pub fn get_errors(&self) -> &Vec<LexError> {
        &self.errors
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
                    span: SpanRange::dummy(15),
                });
            }
        }
    }
}
