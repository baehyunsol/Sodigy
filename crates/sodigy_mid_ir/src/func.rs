use crate::expr::Expr;
use crate::ty::Type;
use sodigy_high_ir::{self as hir, NameBindingType};
use sodigy_parse::IdentWithSpan;
use sodigy_uid::Uid;
use std::collections::HashMap;

mod endec;
mod fmt;
mod graph;
mod lower;

pub use graph::{LocalValueGraph, LocalValueRef};
pub use lower::lower_func;

pub type LocalValueKey = u32;

pub struct Func {
    pub name: IdentWithSpan,

    return_type: Type,
    return_value: Expr,

    // all the local name bindings (names that do not have uids) are
    // stored here
    local_values: HashMap<LocalValueKey, LocalValue>,
    local_values_reachable_from_return_value: HashMap<LocalValueKey, LocalValueRef>,
    pub uid: Uid,
}

pub struct LocalValue {
    pub name: IdentWithSpan,

    // func args and func generics don't have values
    pub value: MaybeInit<hir::Expr, Expr>,

    // iff type annotation for this value exists
    pub ty: MaybeInit<hir::Type, Type>,

    pub parent_func: Uid,
    pub parent_scope: Option<Uid>,
    pub name_binding_type: NameBindingType,

    // ones that created by the compiler vs by the user
    pub is_real: bool,

    // parent.local_values[self.key] = self
    pub key: LocalValueKey,

    // dependency graph on local values
    // it's used for analysis and optimizations
    pub graph: Option<LocalValueGraph>,

    // if this local value is removed by dead code analysis, this flag is set to false
    // DON'T do anything on this value if this flag is false
    pub is_valid: bool,

    // used when traversing graph
    pub visit_flag: VisitFlag,
}

// for `local_values` in `Func`,
// values have to be initialized after `Vec<LocalValue>` is constructed.
// so there must be a placeholder for `hir::Expr`s while `Vec<LocalValue>` is being constructed
//
// it's like `Option<U>`, but has a tmp place for T
pub enum MaybeInit<T, U> {
    None,  // no value at all
    Uninit(T),
    Init(U),
}

impl<T, U> MaybeInit<T, U> {
    pub fn try_unwrap_init(&self) -> Option<&U> {
        match self {
            MaybeInit::Init(v) => Some(v),
            _ => None,
        }
    }

    pub fn try_unwrap_uninit(&self) -> Option<&T> {
        match self {
            MaybeInit::Uninit(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(PartialEq)]
pub enum VisitFlag {
    Visited,
    NotVisited,
    Gray,
}
