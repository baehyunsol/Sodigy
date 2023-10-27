use crate::{ArgDef, Expr, GenericDef, IdentWithSpan, TypeDef};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;

pub struct Stmt {
    pub kind: StmtKind,

    // `span` includes the entire definition
    // e.g. from a keyword `def` to an expression for a function
    pub span: SpanRange,
}

pub enum StmtKind {
    Func(FuncDef),
    Module(IdentWithSpan),
    Use(Use),
    Enum(EnumDef),
    Struct(StructDef),
    Decorator(Decorator),

    // consecutive doc comments are not merged yet
    DocComment(InternedString),
}

pub struct FuncDef {
    pub name: IdentWithSpan,
    pub generics: Vec<GenericDef>,
    pub args: Option<Vec<ArgDef>>,
    pub ret_type: Option<TypeDef>,
    pub ret_val: Expr,
}

pub enum Use {

    // use a;
    Unit(Vec<IdentWithSpan>),

    // use a as b;
    Alias {
        from: Vec<IdentWithSpan>,
        to: IdentWithSpan,
    },

    // use { .. };
    Group {
        pre: Vec<IdentWithSpan>,
        mods: Vec<Use>,
    },
}

// attributes of enums and structs are collected later
// in ast level, it only collects attributes of variants and fields
pub enum Attribute {
    DocComment(String),
    Decorator(Decorator),
}

pub struct EnumDef {
    pub name: IdentWithSpan,
    pub generics: Vec<GenericDef>,
    pub variants: Vec<VariantDef>,
}

pub enum VariantKind {
    Empty,
    Tuple(Vec<TypeDef>),
    Struct(Vec<FieldDef>),
}

pub struct VariantDef {
    pub name: IdentWithSpan,
    pub args: VariantKind,
    pub attributes: Vec<Attribute>,
}

pub struct StructDef {
    pub name: IdentWithSpan,
    pub generics: Vec<GenericDef>,
    pub fields: Vec<FieldDef>,
}

pub struct FieldDef {
    pub name: IdentWithSpan,
    pub ty: TypeDef,
    pub attributes: Vec<Attribute>,
}

#[derive(Clone)]
pub struct Decorator {
    pub name: Vec<IdentWithSpan>,
    pub args: Option<Vec<Expr>>,
}