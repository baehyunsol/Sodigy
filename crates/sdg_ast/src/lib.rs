#![feature(box_patterns)]

mod ast;
mod err;
mod expr;
mod lexer;
mod parse;
mod path;
mod pattern;
mod session;
mod span;
mod stmt;
mod token;
mod utils;
mod value;
mod warning;

#[cfg(test)]
mod tests;

pub use err::SodigyError;
pub use session::{GlobalParseSession, LocalParseSession};
pub use ast::AST;

use err::ParseError;
use lexer::lex_tokens;
use span::Span;
use stmt::parse_stmts;
use token::TokenList;

/// If it returns `Err(())`, the actual errors are in `session`.
pub fn parse_file(s: &[u8], session: &mut LocalParseSession) -> Result<AST, ()> {
    let tokens = lex_tokens(s, session).map_err(|e| session.try_add_error::<(), ParseError>(Err(e)))?;
    let mut tokens = TokenList::from_vec(tokens, Span::new(session.curr_file, 0, 0));
    let stmts = parse_stmts(&mut tokens, session)?;

    AST::from_stmts(stmts, session)
}
