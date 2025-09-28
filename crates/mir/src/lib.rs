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
    pub fn lower(&mut self) -> Result<Block, ()> {
        todo!()
    }
}

// TODO
#[derive(Clone, Debug)]
pub struct Type;
