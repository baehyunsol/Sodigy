use crate::expr::{Expr, ExprKind, InfixOp};
use crate::module::ModulePath;
use crate::session::InternedString;
use crate::span::Span;
use crate::token::{Token, TokenKind};
use crate::value::ValueKind;
use std::slice::Iter;

#[derive(Clone)]
pub struct Use {
    pub path: ModulePath,
    pub alias: InternedString,

    pub span: Span,  // if points to `u` of `use`
}

impl Use {
    pub fn new(path: Vec<InternedString>, alias: InternedString, span: Span) -> Self {
        assert!(!path.is_empty(), "Internal Compiler Error C564E4A");

        Use {
            path: ModulePath::from_names(path),
            alias, span
        }
    }

    pub fn push_front(mut self, path: &Vec<InternedString>) -> Self {
        self.path.push_front(&path);

        self
    }

    // used for resolving names.
    // Exprs generated by this function do not have spans: be careful
    // not to generate errors from Exprs from this
    pub fn to_path(&self) -> ExprKind {
        to_path_impl(self.path.as_ref())
    }

    pub fn iter_path(&self) -> Iter<InternedString> {
        self.path.as_ref().iter()
    }
}

pub fn use_case_to_tokens(Use { path, alias, span }: Use) -> Vec<Token> {
    // `use`, PATH, `as`, ALIAS, `;`
    let mut tokens = Vec::with_capacity(path.len() * 2 + 3);

    tokens.push(Token {
        span,
        kind: TokenKind::keyword_use(),
    });

    for token in path.tokens(span) {
        tokens.push(token);
    }

    tokens.push(Token {
        span,
        kind: TokenKind::keyword_as(),
    });

    tokens.push(Token {
        span,
        kind: TokenKind::Identifier(alias),
    });

    tokens.push(Token {
        span,
        kind: TokenKind::semi_colon(),
    });

    tokens
}

macro_rules! new_path {
    ($p0: expr) => {
        ExprKind::Value(ValueKind::Identifier($p0))
    };
    (recurs, $ps: expr, $pt: expr) => {
        ExprKind::Infix(
            InfixOp::Path,
            Box::new(Expr {
                kind: $ps,
                span: Span::dummy()
            }),
            Box::new(Expr {
                kind: new_path!($pt),
                span: Span::dummy()
            }),
        )
    };
    ($p0: expr, $p1: expr) => {
        new_path!(recurs, new_path!($p0), $p1)
    };
    ($p0: expr, $p1: expr, $p2: expr) => {
        new_path!(recurs, new_path!($p0, $p1), $p2)
    };
    ($p0: expr, $p1: expr, $p2: expr, $p3: expr) => {
        new_path!(recurs, new_path!($p0, $p1, $p2), $p3)
    };
    ($p0: expr, $p1: expr, $p2: expr, $p3: expr, $p4: expr) => {
        new_path!(recurs, new_path!($p0, $p1, $p2, $p3), $p4)
    };
}

// best optimization I can think of
fn to_path_impl(path: &[InternedString]) -> ExprKind {
    assert!(!path.is_empty(), "Internal Compiler Error AAC0E14");

    if path.len() < 4 {

        if path.len() == 1 {
            new_path!(path[0])
        }

        else if path.len() == 2 {
            new_path!(path[0], path[1])
        }

        else {
            new_path!(path[0], path[1], path[2])
        }

    }

    else {

        if path.len() == 4 {
            new_path!(path[0], path[1], path[2], path[3])
        }

        else if path.len() == 5 {
            new_path!(path[0], path[1], path[2], path[3], path[4])
        }

        else {
            new_path!(recurs, to_path_impl(&path[0..(path.len() - 1)]), path[path.len() - 1])
        }

    }
}
