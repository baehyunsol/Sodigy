use crate::ast::NameOrigin;
use crate::expr::{Expr, ExprKind};
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::stmt::ArgDef;
use crate::token::{OpToken, TokenKind};
use hmath::{BigInt, Ratio};
use sdg_uid::UID;

#[derive(Clone)]
pub enum ValueKind {
    Identifier(InternedString, NameOrigin),
    Integer(BigInt),
    Real(Ratio),
    String(Vec<u32>),
    Bytes(Vec<u8>),
    Format(Vec<Expr>),
    List(Vec<Expr>),

    // for a single-element tuple, use a trailing comma
    Tuple(Vec<Expr>),

    // '\' '{' (ARGS ',')? VALUE '}'
    // `ARGS` of lambda and `ARGS` of FuncDef are identical
    Lambda(Vec<ArgDef>, Box<Expr>),

    // BLOCK: '{' DEFS ';' VALUE '}'
    // DEF: PATTERN '=' VALUE
    // DEFs are seperated by ';'
    Block {
        defs: Vec<BlockDef>,
        value: Box<Expr>,
        id: UID,
    },
}

impl ValueKind {
    pub fn get_first_token(&self) -> TokenKind {
        match self {
            ValueKind::Identifier(i, _) => TokenKind::Identifier(*i),
            ValueKind::Integer(n) => TokenKind::Number(n.clone().into()),
            ValueKind::Real(n) => TokenKind::Number(n.clone()),
            ValueKind::String(_) => TokenKind::String(vec![]),
            ValueKind::Bytes(_) => TokenKind::Bytes(vec![]),
            ValueKind::Format(_) => TokenKind::FormattedString(vec![]),
            ValueKind::Lambda(_, _) => TokenKind::Operator(OpToken::BackSlash),
            ValueKind::List(_) => TokenKind::Operator(OpToken::OpeningSquareBracket),
            ValueKind::Tuple(_) => TokenKind::Operator(OpToken::OpeningParenthesis),
            ValueKind::Block { .. } => TokenKind::Operator(OpToken::OpeningCurlyBrace),
        }
    }

    // `{x = 3; y = 4; x + y}` -> `{x = 3; y = 4; x + y}`
    // `{x + y}` -> `x + y`
    pub fn block_to_expr_kind(self) -> ExprKind {
        if let ValueKind::Block { defs, value, .. } = &self {
            if defs.is_empty() {
                value.kind.clone()
            } else {
                ExprKind::Value(self)
            }
        } else {
            panic!(
                "Internal Compiler Error 32D704D714E: {}",
                self.render_err()
            );
        }
    }

    pub fn render_err(&self) -> String {
        self.get_first_token()
            .render_err(&LocalParseSession::dummy())
    }
}

impl From<(InternedString, NameOrigin)> for ValueKind {
    fn from((name, origin): (InternedString, NameOrigin)) -> Self {
        ValueKind::Identifier(name, origin)
    }
}

#[derive(Clone)]
pub struct BlockDef {
    pub(crate) name: InternedString,
    pub(crate) ty: Option<Expr>,
    pub(crate) value: Expr,

    // points to the first character of the name
    pub(crate) span: Span,
}
