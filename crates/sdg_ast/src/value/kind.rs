use crate::ast::NameOrigin;
use crate::expr::{Expr, ExprKind};
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::stmt::ArgDef;
use crate::token::{OpToken, TokenKind};
use crate::utils::{bytes_to_string, v32_to_string};
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

    pub fn is_list(&self) -> bool {
        match self {
            ValueKind::List(_) => true,
            _ => false,
        }
    }

    pub fn is_tuple(&self) -> bool {
        match self {
            ValueKind::Tuple(_) => true,
            _ => false,
        }
    }

    pub fn render_err(&self) -> String {
        self.get_first_token()
            .render_err(&LocalParseSession::dummy())
    }

    pub fn to_string(&self, session: &LocalParseSession) -> String {
        match self {
            ValueKind::Integer(n) => n.to_string(),
            ValueKind::Real(n) => n.to_string(),
            ValueKind::Identifier(ind, _) => bytes_to_string(&session.unintern_string(*ind)),
            ValueKind::String(buf) => format!(
                "{:?}",
                v32_to_string(buf)
                    .expect("Internal Compiler Error 5F6D16DDCB7: {buf:?}"),
            ),
            ValueKind::Bytes(b) => format!(
                "Bytes({})",
                b.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(","),
            ),
            ValueKind::List(elements) | ValueKind::Tuple(elements) | ValueKind::Format(elements) => {
                let (name, opening, closing) = if self.is_list() {
                    ("", "[", "]")
                } else if self.is_tuple() {
                    ("Tuple", "(", ")")
                } else {
                    ("Format", "(", ")")
                };

                format!(
                    "{name}{opening}{}{closing}",
                    elements
                        .iter()
                        .map(|element| element.to_string(session))
                        .collect::<Vec<String>>()
                        .join(",")
                )
            },
            ValueKind::Lambda(args, value) => {
                let args = args
                    .iter()
                    .map(|ArgDef { name, ty, .. }| if let Some(ty) = ty {
                            format!(
                                "{}:{},",
                                bytes_to_string(&session.unintern_string(*name)),
                                ty.to_string(session),
                            )
                        } else {
                            format!("{},", bytes_to_string(&session.unintern_string(*name)))
                        }
                    )
                    .collect::<Vec<String>>()
                    .concat();

                format!("Lambda({args}{})", value.to_string(session))
            },
            ValueKind::Block { defs, value, .. } => {
                let defs = defs
                    .iter()
                    .map(|BlockDef{ name, ty, value, .. }| {
                        format!(
                            "{}{}={};",
                            bytes_to_string(&session.unintern_string(*name)),
                            if let Some(ty) = ty {
                                format!(":{}", ty.to_string(session))
                            } else {
                                String::new()
                            },
                            value.to_string(session),
                        )
                    })
                    .collect::<Vec<String>>()
                    .concat();

                format!("{}{defs}{}{}", '{', value.to_string(session), '}',)
            }
        }
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
