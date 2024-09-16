use sodigy_high_ir::NameBindingType;
use sodigy_intern::InternedNumeric;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

pub struct Expr {
    kind: ExprKind,
    span: SpanRange,

    // TODO: what if it does not have a type annotation?
    // ty: Type,
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
        func: Box<Expr>,
        args: Vec<Expr>,
        tail_call: bool,
    },
}
