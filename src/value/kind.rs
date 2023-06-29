use crate::expr::Expr;
use crate::token::{OpToken, TokenKind};
use hmath::{BigInt, Ratio};

#[derive(Clone)]
pub enum ValueKind {

    // TODO: How about `True`, `False`, and `None`? Do we treat them like other identifiers?
    Identifier(u32),

    Integer(BigInt),
    Real(Ratio),
    String(u32),
    List(Vec<Box<Expr>>),
}

impl ValueKind {

    pub fn is_identifier(&self) -> bool {

        if let ValueKind::Identifier(_) = self {
            true
        }

        else {
            false
        }

    }

    pub fn get_first_token(&self) -> TokenKind {

        match self {
            ValueKind::Identifier(i) => TokenKind::Identifier(*i),
            ValueKind::Integer(n) => TokenKind::Number(n.clone().into()),
            ValueKind::Real(n) => TokenKind::Number(n.clone()),
            ValueKind::String(i) => TokenKind::String(*i),
            ValueKind::List(_) => TokenKind::Operator(OpToken::OpeningSquareBracket),
        }

    }

}