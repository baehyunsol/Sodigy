use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::token::{Token, TokenKind};

#[derive(Clone)]
pub struct ModulePath (Vec<InternedString>);

impl ModulePath {
    pub fn empty() -> Self {
        ModulePath(vec![])
    }

    pub fn from_names(names: Vec<InternedString>) -> Self {
        ModulePath(names)
    }

    pub fn push_front(&mut self, path: &Vec<InternedString>) {
        self.0 = vec![
            path.clone(),
            self.0.clone(),
        ].concat();
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn tokens(&self, span: Span) -> Vec<Token> {
        self.0.iter().map(
            |s| Token {
                span,
                kind: TokenKind::Identifier(*s),
            }
        ).collect()
    }

    pub fn as_ref(&self) -> &Vec<InternedString> {
        &self.0
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        self.0.iter().map(|s| s.to_string(session)).collect::<Vec<String>>().join(".")
    }
}
