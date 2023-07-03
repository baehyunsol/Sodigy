use crate::span::Span;

mod arg_def;
mod decorator;
mod func_def;
mod kind;
mod parse;

pub use arg_def::{parse_arg_def, ArgDef};
pub use decorator::Decorator;
pub use func_def::FuncDef;
pub use kind::StmtKind;

pub struct Stmt {
    kind: StmtKind,
    span: Span,
}
