#![feature(box_patterns)]

mod err;
mod expr;
mod lexer;
mod module;
mod parse;
mod session;
mod span;
mod stmt;
mod token;
mod utils;
mod value;

pub use err::ParseError;
pub use session::{GlobalParseSession, LocalParseSession};
pub use stmt::Stmt;

use lexer::lex_tokens;
use stmt::parse_stmts;
use token::TokenList;

/// 
pub fn parse_file(s: &[u8], session: &mut LocalParseSession) -> Result<Vec<Stmt>, ParseError> {
    let tokens = lex_tokens(s, session)?;
    let mut tokens = TokenList::from_vec(tokens);

    parse_stmts(&mut tokens)
}