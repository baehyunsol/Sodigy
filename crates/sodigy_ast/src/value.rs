use crate::{ArgDef, expr::Expr, ScopeBlock, TypeDef};
use sodigy_intern::{InternedString, InternedNumeric};
use sodigy_uid::Uid;

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

        // in scoped_lets
        // `let add(x: Int, y: Int): Int = x + y;`
        // -> `let add = \{x: Int, y: Int, x + y};`

        // though users cannot annotate ret_type of a lambda,
        // lambdas generated from scoped_lets sometimes require this field
        ret_type: Option<Box<TypeDef>>,
        lowered_from_scoped_let: bool,
    },
    Scope {
        scope: ScopeBlock,
        uid: Uid,
    },
}
