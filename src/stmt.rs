use crate::span::Span;

mod kind;
mod parse;

pub use kind::StmtKind;

pub struct Stmt {
    kind: StmtKind,
    span: Span
}