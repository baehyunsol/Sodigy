use crate::expr::Expr;
use crate::session::{InternedString, LocalParseSession};
use crate::stmt::ArgDef;
use crate::token::{OpToken, TokenKind};
use hmath::{BigInt, Ratio};

#[derive(Clone)]
pub enum ValueKind {
    Identifier(InternedString),
    Integer(BigInt),
    Real(Ratio),
    String(InternedString),
    List(Vec<Box<Expr>>),

    // '\' '{' (ARGS ',')? VALUE '}'
    // `ARGS` of lambda and `ARGS` of FuncDef are identical
    Lambda(Vec<Box<ArgDef>>, Box<Expr>),

    // BLOCK: '{' DEFS ';' VALUE '}'
    // DEF: PATTERN '=' VALUE
    // DEFs are seperated by ';'
    Block {
        defs: Vec<(InternedString, Box<Expr>)>, // (name, value)
        value: Box<Expr>,
    },
}

impl ValueKind {
    pub fn is_identifier(&self) -> bool {
        if let ValueKind::Identifier(_) = self {
            true
        } else {
            false
        }
    }

    pub fn get_first_token(&self) -> TokenKind {
        match self {
            ValueKind::Identifier(i) => TokenKind::Identifier(*i),
            ValueKind::Integer(n) => TokenKind::Number(n.clone().into()),
            ValueKind::Real(n) => TokenKind::Number(n.clone()),
            ValueKind::String(i) => TokenKind::String(*i),
            ValueKind::Lambda(_, _) => TokenKind::Operator(OpToken::BackSlash),
            ValueKind::List(_) => TokenKind::Operator(OpToken::OpeningSquareBracket),
            ValueKind::Block { .. } => TokenKind::Operator(OpToken::OpeningCurlyBrace),
        }
    }

    pub fn render_err(&self) -> String {
        self.get_first_token()
            .render_err(&LocalParseSession::dummy())
    }
}