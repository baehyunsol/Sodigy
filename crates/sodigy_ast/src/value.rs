use sodigy_intern::{InternedString, InternedNumeric};
use crate::{ArgDef, expr::Expr, ScopeDef};

#[derive(Clone)]
pub enum ValueKind {
    // Don't do any kind of analysis/optimization in this stage
    Identifier(InternedString),
    Number(InternedNumeric),
    String {
        s: InternedString,
        is_binary: bool,  // `b` prefix
    },
    Char(char),
    List(Vec<Expr>),
    Tuple(Vec<Expr>),

    // Later optimize
    //   - remove empty strings
    //   - concat consecutive strings
    //   - unwrap format strings without values
    Format(Vec<Expr>),

    // Later inspect -> closures and recursive lambdas
    Lambda {
        args: Vec<ArgDef>,
        value: Box<Expr>,
    },

    // Later
    //   - optimize 0/1 used defs
    //   - find recursive defs
    //   - unwrap scopes without defs
    Scope(ScopeDef),
}
