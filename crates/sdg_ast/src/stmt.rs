use crate::span::Span;

mod arg_def;
mod decorator;
mod func_def;
mod kind;
mod parse;

#[cfg(test)]
mod tests;

mod use_;

pub use arg_def::{parse_arg_def, ArgDef, GetNameOfArg};
pub use decorator::Decorator;
pub use func_def::FuncDef;
pub use kind::StmtKind;
pub use parse::parse_stmts;
pub use use_::{Use, use_case_to_tokens};

#[cfg(test)]
pub use parse::parse_use;

pub struct Stmt {
    pub(crate) kind: StmtKind,
    span: Span,
}