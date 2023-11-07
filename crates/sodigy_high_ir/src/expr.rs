use crate::func::Arg;
use crate::names::IdentWithOrigin;
use crate::pattern::Pattern;
use sodigy_ast::{DottedNames, InfixOp, PostfixOp, PrefixOp};
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
    Identifier(IdentWithOrigin),
    Integer(InternedNumeric),
    Ratio(InternedNumeric),
    Char(char),
    String {
        s: InternedString,
        is_binary: bool,  // `b` prefix
    },

    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },

    List(Vec<Expr>),
    Tuple(Vec<Expr>),
    Format(Vec<Expr>),

    Scope(Scope),
    Match(Match),
    Lambda(Lambda),
    Branch(Branch),

    // `a.b.c` -> `Path { head: a, tail: [b, c] }`
    Path {
        head: Box<Expr>,
        tail: DottedNames,
    },

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

pub struct Match {
    arms: Vec<MatchArm>,
    value: Box<Expr>,
}

pub struct MatchArm {
    pub pattern: Pattern,
    pub value: Expr,
    pub guard: Option<Expr>,
}

pub struct Lambda {
    pub args: Vec<Arg>,
    pub value: Box<Expr>,
    pub foreign_names: Vec<IdentWithOrigin>,
    pub uid: Uid,
}

pub struct Branch {
    pub arms: Vec<BranchArm>,
}

pub struct BranchArm {
    pub cond: Option<Expr>,
    pub let_bind: Option<Expr>,
    pub value: Expr,
}
