#![deny(unused_imports)]

use sodigy_high_ir::NameBindingType;
use sodigy_intern::InternedNumeric;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

// TODO: mir
// it assumes that name resolution (which is not implemented yet) is complete

pub struct Expr {
    kind: ExprKind,
    span: SpanRange,
    ty: Type,
}

pub enum ExprKind {
    // `Ratio`s are lowered to function calls
    Integer(InternedNumeric),
    LocalValue {
        binding_type: NameBindingType,
        origin: Uid,
        index: usize,
    },
    Object(Uid),
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        tail_call: bool,
    },
}

pub struct Type {
    // TODO: ...
}
