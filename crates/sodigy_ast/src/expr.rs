use crate::{BranchArm, IdentWithSpan, MatchArm, ops::{InfixOp, PostfixOp, PrefixOp}};
use crate::value::ValueKind;
use sodigy_span::SpanRange;

#[derive(Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: SpanRange,
}

/****************************************
 *  spans of exprs                      *
 *  value: see `value.rs`               *
 *  pre/post/infix op: the operator     *
 *  path: `.`                           *
 *  call: the parenthesis               *
 *  branch: keyword `if`                *
 *  match: keyword `match`              *
 ****************************************/

#[derive(Clone)]
pub enum ExprKind {
    Value(ValueKind),
    PrefixOp(PrefixOp, Box<Expr>),
    PostfixOp(PostfixOp, Box<Expr>),
    InfixOp(InfixOp, Box<Expr>, Box<Expr>),

    // a.b
    Path { pre: Box<Expr>, post: IdentWithSpan },
    Call { functor: Box<Expr>, args: Vec<Expr> },
    Branch(Vec<BranchArm>),

    // Don't do anything in this stage
    Match { value: Box<Expr>, arms: Vec<MatchArm> },
}
