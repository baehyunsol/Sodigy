use crate::{BranchArm, IdentWithSpan, MatchArm, ops::{InfixOp, PostfixOp, PrefixOp}, StructInitDef};
use crate::value::ValueKind;
use sodigy_span::SpanRange;

mod fmt;

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: SpanRange,
}

impl Expr {
    // `{x}` -> `x`
    pub fn peel_unnecessary_brace(&mut self) {
        match &self.kind {
            ExprKind::Value(ValueKind::Scope { scope, .. }) if scope.has_no_lets() => {
                *self = *scope.value.clone();
            },
            _ => { /* nop */ },
        }
    }
}

/****************************************
 *  spans of exprs                      *
 *  value: see `value.rs`               *
 *  pre/post/infix op: the operator     *
 *  path: `.`                           *
 *  call: the parenthesis               *
 *  branch: the first `if` keyword      *
 *  match: `match` keyword              *
 ****************************************/

#[derive(Clone, Debug)]
pub enum ExprKind {
    Value(ValueKind),
    PrefixOp(PrefixOp, Box<Expr>),
    PostfixOp(PostfixOp, Box<Expr>),
    InfixOp(InfixOp, Box<Expr>, Box<Expr>),

    // `a.b`: `Path { pre: a, post: b }`
    Path { pre: Box<Expr>, post: IdentWithSpan },
    Call { func: Box<Expr>, args: Vec<Expr> },

    // foo { bar: 3, baz: 4 }
    StructInit {
        struct_: Box<Expr>,
        fields: Vec<StructInitDef>,
    },

    // Better be defined in a recursive way?
    Branch(Vec<BranchArm>),

    Match {
        value: Box<Expr>,
        arms: Vec<MatchArm>,
        is_lowered_from_if_pattern: bool,
    },

    // It doesn't do anything in runtime. It's just for diagnosis.
    Parenthesis(Box<Expr>),

    // placeholder for erroneous exprs
    Error,
}
