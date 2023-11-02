use sodigy_intern::{InternedString, InternedNumeric};
use sodigy_uid::Uid;
use crate::{ArgDef, expr::Expr, ScopeDef};

#[derive(Clone)]
pub enum ValueKind {
    Identifier(InternedString),
    Number(InternedNumeric),
    String {
        s: InternedString,
        is_binary: bool,  // `b` prefix
    },
    Char(char),
    List(Vec<Expr>),
    Tuple(Vec<Expr>),
    Format(Vec<Expr>),
    Lambda {
        args: Vec<ArgDef>,
        value: Box<Expr>,
        uid: Uid,
    },
    Scope {
        scope: ScopeDef,
        uid: Uid,
    },
}
