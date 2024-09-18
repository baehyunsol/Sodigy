use crate::expr::Expr;
use crate::ty::Type;
use sodigy_high_ir::{self as hir, NameBindingType};
use sodigy_parse::IdentWithSpan;
use sodigy_uid::Uid;

mod lower;

pub struct Func {
    name: IdentWithSpan,

    return_type: Type,
    return_value: Expr,

    // all the local name bindings (names that do not have uids) are
    // stored here
    local_values: Vec<LocalValue>,
    uid: Uid,
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

    // parent.local_values[self.index] = self
    pub index: usize,
}

// for `local_values` in `Func`,
// values have to be initialized after `Vec<LocalValue>` is constructed.
// so there must be a placeholder for `hir::Expr`s while `Vec<LocalValue>` is being constructed
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
