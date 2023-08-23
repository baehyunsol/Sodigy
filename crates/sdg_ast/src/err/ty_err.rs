use crate::expr::Expr;

pub struct TypeError {
    kind: TypeErrorKind,
    expected: Option<Expr>,
    got: Option<Expr>,
}

enum TypeErrorKind {
    /// when a condition of a branch is not Boolean
    BranchCondition,

    /// in case of `if X { A } else if Y { B } else { C }`,
    /// where `A` and `B` have the same type but `C` hasn't,
    /// it's NthBranch(2).
    NthBranch(usize),

    /// when `foo` in `foo()` is not callable
    NotCallable,

    MissingFuncArg,
    UnexpectedFuncArg,
    WrongFuncArg,
}
