use crate::expr::Expr;
use sodigy_ast::{DottedNames, IdentWithSpan};

mod endec;
mod fmt;
mod lower;

pub use lower::lower_ast_attributes;

#[derive(Clone)]
pub enum Attribute {
    DocComment(IdentWithSpan),
    Decorator(Decorator),
}

#[derive(Clone)]
pub struct Decorator {
    pub name: DottedNames,
    pub args: Option<Vec<Expr>>,
}
