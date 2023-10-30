use crate::names::NameOrigin;
use crate::pattern::Pattern;
use sodigy_ast::{InfixOp, PostfixOp, PrefixOp};
use sodigy_intern::{InternedNumeric, InternedString};
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

mod lower;

pub use lower::lower_ast_expr;

pub struct Expr {
    kind: ExprKind,
    span: SpanRange,
}

pub enum ExprKind {
    Identifier(InternedString, NameOrigin),

    // TODO: any other info?
    Integer(InternedNumeric),
    Ratio(InternedNumeric),
    String {
        s: InternedString,
        is_binary: bool,  // `b` prefix
    },

    Scope(Scope),

    PrefixOp(PrefixOp, Box<Expr>),
    PostfixOp(PostfixOp, Box<Expr>),
    InfixOp(InfixOp, Box<Expr>, Box<Expr>),
}

pub struct Scope {
    pub defs: Vec<LocalDef>,
    pub value: Box<Expr>,
    pub uid: Uid,
}

pub struct LocalDef {
    pub pattern: Pattern,
    pub value: Expr,
    pub let_span: SpanRange,
}
