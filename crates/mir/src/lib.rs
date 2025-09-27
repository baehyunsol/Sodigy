mod block;
mod expr;
mod func;
mod r#if;
mod r#let;

pub use block::Block;
pub use expr::Expr;
pub use func::Func;
pub use r#if::If;
pub use r#let::Let;

/// It's used to analyse the code in various ways.
pub struct RefCount {
    pub conditional: usize,
    pub unconditional: usize,
}
