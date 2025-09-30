use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::{HashMap, HashSet};

pub enum Namespace {
    FuncArg {
        names: HashMap<InternedString, (Span, NameKind, u32 /* count */)>,
        index: HashMap<InternedString, usize>,
    },
    Block {
        names: HashMap<InternedString, (Span, NameKind, u32 /* count */)>,
    },
    FuncDef {
        name: InternedString,
        foreign_names: HashSet<(InternedString, Span /* def_span */)>,
    },
}

// impl Namespace {
//     pub fn new(kind: NamespaceKind, names: HashMap<InternedString, (Span, NameKind)>) -> Self {
//         Namespace { kind, names }
//     }
// }

pub enum NamespaceKind {
    Prelude,
    FuncArg,
    Block,  // declarations in a block
    Local,  // anything other than those
}

#[derive(Clone, Copy, Debug)]
pub struct IdentWithOrigin {
    pub id: InternedString,
    pub span: Span,
    pub origin: NameOrigin,

    // It's used to uniquely identify the identifiers.
    pub def_span: Span,
}

#[derive(Clone, Copy, Debug)]
pub enum NameOrigin {
    // If funcs are nested, only the inner-most function counts.
    FuncArg {
        index: usize,
    },
    // Local value that's declared inside the same function (inner-most).
    Local {
        kind: NameKind,
    },
    // If this identifier is not declared inside the same function, it's Foreign.
    Foreign {
        kind: NameKind,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum NameKind {
    Let,
    Func,
    Struct,
    Enum,
    Module,
    Use,
    FuncArg,
}
