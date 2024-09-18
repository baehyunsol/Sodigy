use crate::ty::Type;
use sodigy_intern::InternedNumeric;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

mod lower;

pub struct Expr {
    kind: ExprKind,
    span: SpanRange,

    // TODO: replace `Type` with `&'session Type` when the other implementation is complete
    ty: Option<Type>,
}

pub enum ExprKind {
    // `Ratio`, `Char` and `String` are lowered to function calls
    Integer(InternedNumeric),
    LocalValue {
        // uid of the function it belongs to, not a local scope
        origin: Uid,

        // index in the function, not in a local scope
        index: usize,
    },
    Object(Uid),
    Call {
        func: MirFunc,
        args: Vec<Expr>,
        tail_call: bool,
    },
}

pub enum MirFunc {
    Static(Uid),
    Dynamic(Box<Expr>),
}
