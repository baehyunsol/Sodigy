use crate::expr::Expr;
use sodigy_ast::DottedNames;
use sodigy_parse::IdentWithSpan;

mod endec;
mod fmt;
mod lower;

pub use lower::lower_ast_attributes;

// TODO: if we declare `Expr` as a generic, we can reuse this
//       currently, it's defined 3 times (ast, hir, and mir) and that's a waste
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
