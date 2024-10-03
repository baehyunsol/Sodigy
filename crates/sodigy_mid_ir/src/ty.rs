use crate::expr::Expr;
use sodigy_uid::Uid;

mod endec;
mod fmt;
mod lower;

pub use lower::lower_ty;

#[derive(Clone)]
pub enum Type {
    // when user omits a type annotation
    HasToBeInferred,

    // TODO: `Box<Expr>` is too expensive
    HasToBeLowered(Box<Expr>),

    // Int, Bool, Type, ...
    // TODO: study type theories and find how it's called
    Simple(Uid),
}

impl Type {
    pub fn from_uid(uid: Uid) -> Self {
        Type::Simple(uid)
    }
}
