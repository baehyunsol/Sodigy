use sodigy_error::Error;
use sodigy_file::File;
use sodigy_span::Span;
use sodigy_token::Token;

mod attribute;
mod block;
mod check;
mod r#enum;
mod expr;
mod func;
mod r#if;
mod r#let;
mod module;
mod pattern;
mod r#struct;
mod tokens;
mod r#use;

pub use attribute::{Attribute, Decorator, DocComment};
pub use block::Block;
pub use r#enum::Enum;
pub use expr::Expr;
pub use func::{CallArg, Func, FuncArgDef};
pub use r#if::If;
pub use r#let::Let;
pub use module::Module;
pub use pattern::Pattern;
pub use r#struct::{Struct, StructInitField};
pub(crate) use tokens::Tokens;
pub use r#use::Use;

pub fn parse(tokens: &[Token], file: File) -> Result<Block, Vec<Error>> {
    let mut tokens = Tokens::new(tokens, tokens.last().map(|t| t.span.end()).unwrap_or(Span::None));
    let block = tokens.parse_block(
        true, // top-level
        Span::file(file),
    )?;

    block.check(true /* top_level */)?;
    Ok(block)
}
