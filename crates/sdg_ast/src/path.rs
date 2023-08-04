// not file path, but Sodigy path, like `a.b.c`

use crate::ast::{ASTError, NameScope};
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::token::{Token, TokenKind};

#[derive(Clone, Default)]
pub struct Path(Vec<(InternedString, Span)>);

impl Path {
    pub fn empty() -> Self {
        Path(vec![])
    }

    pub fn from_names(names: Vec<(InternedString, Span)>) -> Self {
        Path(names)
    }

    pub fn append_front(&mut self, path: &Vec<(InternedString, Span)>) {
        self.0 = vec![
            path.clone(),
            self.0.clone(),
        ].concat();
    }

    pub fn push(&mut self, path: (InternedString, Span)) {
        self.0.push(path);
    }

    pub fn get_name_by_index(&self, index: usize) -> InternedString {
        self.0[index].0
    }

    pub fn get_span_by_index(&self, index: usize) -> Span {
        self.0[index].1
    }

    pub fn slice_from(&self, index: usize) -> &[(InternedString, Span)] {
        &self.0[index..]
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn tokens(&self) -> Vec<Token> {
        self.0.iter().map(
            |(name, span)| Token {
                span: *span,
                kind: TokenKind::Identifier(*name),
            }
        ).collect()
    }

    pub fn resolve_names(&mut self, name_scope: &NameScope, session: &mut LocalParseSession) {
        match name_scope.search_name(self.get_name_by_index(0)) {
            Ok((Some(u), _)) => {
                if self.len() == 1 {
                    *self = Path::from_names(
                        u.iter_path().map(
                            |(n, s)| (*n, *s)
                        ).collect()
                    );
                } else {
                    *self = Path::from_names(
                        u.iter_path().chain(self.slice_from(1).iter()).map(
                            |(n, s)| (*n, *s)
                        ).collect()
                    );
                }
            },
            Ok((None, _)) => {},
            Err(_) => {
                session.add_error(ASTError::no_def(
                    self.get_name_by_index(0),
                    self.get_span_by_index(0),
                    name_scope.clone(),
                ));
            }
        }
    }

    pub fn as_ref(&self) -> &Vec<(InternedString, Span)> {
        &self.0
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        self.0.iter().map(|(s, _)| s.to_string(session)).collect::<Vec<String>>().join(".")
    }
}
