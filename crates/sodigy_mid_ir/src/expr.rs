use crate::func::LocalValueKey;
use crate::ty::Type;
use sodigy_intern::InternedNumeric;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

mod endec;
mod fmt;
mod lower;

pub use lower::lower_expr;

#[derive(Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: SpanRange,

    // TODO: is it inferred type or an annotated type?
    // TODO: replace `Type` with `&'session Type` when the other implementation is complete
    ty: Option<Type>,
}

#[derive(Clone)]
pub enum ExprKind {
    // `Ratio`, `Char` and `String` are lowered to function calls
    Integer(InternedNumeric),
    LocalValue {
        // uid of the function it belongs to, not a local scope
        origin: Uid,
        key: LocalValueKey,
    },
    Object(Uid),
    Call {
        func: MirFunc,
        args: Vec<Expr>,
        tail_call: bool,
    },
}

#[derive(Clone)]
pub enum MirFunc {
    Static(Uid),
    Dynamic(Box<Expr>),
}
