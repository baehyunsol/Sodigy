use sodigy_hir as hir;

mod block;
mod expr;
mod func;
mod r#if;
mod r#let;
mod session;

pub use block::Block;
pub use expr::Expr;
pub use func::Func;
pub use r#if::If;
pub use r#let::Let;
pub use session::Session;

impl Session {
    /// Errors and warnings are stored in the session.
    pub fn lower(&mut self, hir_block: &hir::Block) -> Result<Block, ()> {
        Block::from_hir(hir_block, self)
    }
}
