mod endec;
mod fmt;
mod lower;

pub use lower::lower_ty;

#[derive(Clone)]
pub enum Type {
    // when user omits a type annotation
    HasToBeInferred,
}
