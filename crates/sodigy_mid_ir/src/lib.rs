#![deny(unused_imports)]

// rough sketch
pub struct Data {
    kind: DataKind,
    ty: Type,

    // We still need this for error messages
    span: SpanRange,
}

// rough sketch
pub enum DataKind {
    Global(Uid),
    LocalNameBinding,

    // TODO: other than `SodigyNumber`?
    Integer(SodigyNumber),

    // Ratio, Char, String, Call, List, Format, StructInit, PrefixOp, PostfixOp, and InfixOp in HIR are lowered to this kind
    // Paths are lowered to either NameBinding or Call
    Call {
        f: Uid,
        args: Vec<Data>,
    },

    // like Call, but `f` is determined on runtime
    DynCall {
        f: Box<Data>,
        args: Vec<Data>,
    },

    // Branches in HIR are lowered to this kind
    // `&&` and `||` are lowered to this kind
    Branch,
}

// it's a first-class Sodigy object, but we don't want unnecessary bloats (like spans or DataKind::Branch)
// it has to be rich enough represent `Int`, `List(Int)`, `Result(List(Int), ())`, `(Int, Char)` ... how about `List(T)`?
// it's a result of `evaluate(v: Data)`
pub struct Type {
    Solid(Uid),
    Param(Uid, Vec<Type>),
    Generic(/* TODO: how do we represent one? */),
}
