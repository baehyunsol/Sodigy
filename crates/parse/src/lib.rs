use sodigy_error::Error;
use sodigy_span::Span;
use sodigy_token::Token;

mod block;
mod deco;
mod expr;
mod func;
mod r#let;
mod tokens;

pub use block::Block;
pub use deco::{Decorator, DocComment};
pub use expr::Expr;
pub use func::Func;
pub use r#let::Let;
pub(crate) use tokens::Tokens;

pub fn parse(tokens: &[Token]) -> Result<Block, Vec<Error>> {
    let mut tokens = Tokens::new(tokens, tokens.last().map(|t| t.span.end()).unwrap_or(Span::None));
    tokens.parse_block()
}
