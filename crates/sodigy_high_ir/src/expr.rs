use crate::Type;
use crate::func::Arg;
use crate::names::IdentWithOrigin;
use crate::pattern::Pattern;
use crate::scope::{Scope, ScopedLet};
use sodigy_ast::{FieldKind, InfixOp, PostfixOp, PrefixOp};
use sodigy_intern::{InternedNumeric, InternedString};
use sodigy_parse::IdentWithSpan;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

mod endec;
mod fmt;
pub mod lambda;
mod lower;

pub use lower::{lower_ast_expr, try_warn_unnecessary_paren};

#[derive(Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: SpanRange,
}

#[derive(Clone)]
pub enum ExprKind {
    Identifier(IdentWithOrigin),
    Integer(InternedNumeric),
    Ratio(InternedNumeric),
    Char(char),
    String {
        content: InternedString,
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

    Field {
        pre: Box<Expr>,
        post: FieldKind,
    },

    PrefixOp(PrefixOp, Box<Expr>),
    PostfixOp(PostfixOp, Box<Expr>),
    InfixOp(InfixOp, Box<Expr>, Box<Expr>),
}

#[derive(Clone)]
pub struct Match {
    pub arms: Vec<MatchArm>,
    pub value: Box<Expr>,
    pub is_lowered_from_if_pattern: bool,
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

    // see comments in sodigy_ast::value::Lambda
    pub return_type: Option<Box<Type>>,
    pub lowered_from_scoped_let: bool,
}

#[derive(Clone)]
pub struct Branch {
    pub arms: Vec<BranchArm>,
}

#[derive(Clone)]
pub struct BranchArm {
    pub cond: Option<Expr>,
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
