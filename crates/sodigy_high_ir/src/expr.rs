use crate::func::Arg;
use crate::names::IdentWithOrigin;
use crate::pattern::Pattern;
use sodigy_ast::{DottedNames, IdentWithSpan, InfixOp, PostfixOp, PrefixOp};
use sodigy_intern::{InternedNumeric, InternedString};
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

mod fmt;
pub mod lambda;
mod lower;

pub use lower::{lower_ast_expr, try_warn_unnecessary_paren};

#[derive(Clone)]
pub struct Expr {
    pub kind: ExprKind,
    span: SpanRange,
}

#[derive(Clone)]
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

    StructInit(StructInit),

    // `a.b.c` -> `Path { head: a, tail: [b, c] }`
    Path {
        head: Box<Expr>,
        tail: DottedNames,
    },

    PrefixOp(PrefixOp, Box<Expr>),
    PostfixOp(PostfixOp, Box<Expr>),
    InfixOp(InfixOp, Box<Expr>, Box<Expr>),
}

#[derive(Clone)]
pub struct Scope {
    // used later for type-checking
    pub original_patterns: Vec<(Pattern, Expr)>,

    pub defs: Vec<LocalDef>,
    pub value: Box<Expr>,
    pub uid: Uid,
}

#[derive(Clone)]
pub struct LocalDef {
    pub name: IdentWithSpan,
    pub value: Expr,

    // the compiler generates tmp local defs during the compilation
    pub is_real: bool,
}

#[derive(Clone)]
pub struct Match {
    pub arms: Vec<MatchArm>,
    pub value: Box<Expr>,
}

#[derive(Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub value: Expr,
    pub guard: Option<Expr>,
}

#[derive(Clone)]
pub struct Lambda {
    pub args: Vec<Arg>,
    pub value: Box<Expr>,
    pub captured_values: Vec<Expr>,
    pub uid: Uid,
}

#[derive(Clone)]
pub struct Branch {
    pub arms: Vec<BranchArm>,
}

#[derive(Clone)]
pub struct BranchArm {
    pub cond: Option<Expr>,
    pub let_bind: Option<Expr>,
    pub value: Expr,
}

#[derive(Clone)]
pub struct StructInit {
    pub struct_: Box<Expr>,
    pub fields: Vec<StructInitField>,
}

#[derive(Clone)]
pub struct StructInitField {
    pub name: IdentWithSpan,
    pub value: Expr,
}
