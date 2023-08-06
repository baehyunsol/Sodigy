// not file path, but Sodigy path, like `a.b.c`

use crate::ast::{ASTError, NameOrigin, NameScope};
use crate::expr::{Expr, ExprKind, InfixOp};
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::token::{Token, TokenKind};
use crate::value::ValueKind;

#[derive(Clone, Default)]
pub struct Path(Vec<(InternedString, Span)>);

impl Path {
    pub fn empty() -> Self {
        Path(vec![])
    }

    pub fn root(session: &mut LocalParseSession) -> Self {
        Path(vec![
            (session.intern_string(b"root"), Span::dummy()),
        ])
    }

    pub fn from_names(names: Vec<(InternedString, Span)>) -> Self {
        Path(names)
    }

    pub fn append_front(&mut self, path: &[(InternedString, Span)]) {
        self.0 = vec![
            path.to_vec(),
            self.0.clone(),
        ].concat();
    }

    pub fn append_back(&mut self, path: &[(InternedString, Span)]) {
        self.0 = vec![
            self.0.clone(),
            path.to_vec(),
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

    pub fn slice_to(&self, index: usize) -> &[(InternedString, Span)] {
        &self.0[..index]
    }

    pub fn last(&self) -> Option<(InternedString, Span)> {
        self.0.last().map(|(n, s)| (*n, *s))
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

    pub fn try_from_expr(e: &Expr) -> Option<Self> {
        match &e.kind {
            ExprKind::Value(ValueKind::Identifier(name, _)) => Some(Path(vec![(*name, e.span)])),
            // TODO: O(n^2)
            ExprKind::Infix(InfixOp::Path, op1, op2) => {
                let ex1 = Path::try_from_expr(&op1);
                let ex2 = Path::try_from_expr(&op2);

                match (ex1, ex2) {
                    (Some(ex1), Some(ex2)) => Some(Path(vec![
                        ex1.0,
                        ex2.0,
                    ].concat())),
                    _ => None,
                }
            },
            _ => None,
        }
    }

    // TODO: `Use::to_path`'s implementation and this impl are very similar
    pub fn into_expr(&self) -> Expr {
        match self.0.len() {
            0 => unreachable!(
                "Internal Compiler Error 34F9745739F"
            ),
            1 => Expr {
                kind: ExprKind::Value(ValueKind::Identifier(self.0[0].0, NameOrigin::NotKnownYet)),
                span: self.0[0].1,
            },
            2 => Expr {
                kind: ExprKind::Infix(
                    InfixOp::Path,
                    Box::new(Expr {
                        kind: ExprKind::Value(ValueKind::Identifier(self.0[0].0, NameOrigin::NotKnownYet)),
                        span: self.0[0].1,
                    }),
                    Box::new(Expr {
                        kind: ExprKind::Value(ValueKind::Identifier(self.0[1].0, NameOrigin::SubPath)),
                        span: self.0[1].1,
                    }),
                ),
                span: Span::dummy(),
            },
            _ => Expr {
                kind: ExprKind::Infix(
                    InfixOp::Path,
                    Box::new(Path(self.0[..(self.0.len() - 1)].to_vec()).into_expr()),
                    Box::new(Expr {
                        kind: ExprKind::Value(ValueKind::Identifier(self.0[self.0.len() - 1].0, NameOrigin::SubPath)),
                        span: self.0[self.0.len() - 1].1,
                    }),
                ),
                span: Span::dummy(),
            },
        }
    }

    pub fn resolve_names(&mut self, name_scope: &mut NameScope, session: &mut LocalParseSession) {
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
