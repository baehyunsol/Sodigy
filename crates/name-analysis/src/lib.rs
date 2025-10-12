use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

pub enum Namespace {
    ForeignNameCollector {
        is_func: bool,
        foreign_names: HashMap<InternedString, (NameOrigin, Span /* def_span */)>,
    },
    FuncArg {
        names: HashMap<InternedString, (Span, NameKind, UseCount)>,
        index: HashMap<InternedString, usize>,
    },
    Generic {
        names: HashMap<InternedString, (Span, NameKind, UseCount)>,
        index: HashMap<InternedString, usize>,
    },
    Block {
        names: HashMap<InternedString, (Span, NameKind, UseCount)>,
    },
    Pattern {
        names: HashMap<InternedString, (Span, NameKind, UseCount)>,
    },
}

pub enum NamespaceKind {
    Prelude,
    FuncArg,
    Generic,
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
    // If funcs are nested, only the inner-most function counts.
    Generic {
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
    Let { is_top_level: bool },
    Func,
    Struct,
    Enum,
    Module,
    Use,
    FuncArg,
    Generic,
    PatternNameBind,
}

// The compiler has to count how many times each name is used for various reasons.
// For example, if a name is never used, it throws a warning and remove the definition.
// Assertions make the problem tricky, because an assertion may or may not be removed
// according to compiler options. So we count the numbers separately.
#[derive(Clone, Copy, Debug)]
pub struct UseCount {
    pub assert: Counter,

    // everything other than assertions
    pub expr: Counter,
}

impl UseCount {
    pub fn new() -> Self {
        UseCount {
            assert: Counter::Never,
            expr: Counter::Never,
        }
    }

    pub fn is_zero(&self) -> bool {
        matches!(
            self,
            UseCount { assert: Counter::Never, expr: Counter::Never },
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Counter {
    Never,
    Once,
    Multiple,
}

impl Counter {
    pub fn increment(&mut self) {
        match self {
            Counter::Never => {
                *self = Counter::Once;
            },
            _ => {
                *self = Counter::Multiple;
            },
        }
    }
}
