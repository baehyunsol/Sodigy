#![feature(box_patterns)]

mod ast;
mod endec;
mod err;
mod expr;
mod file_system;
mod hash;
mod lexer;
mod module;
mod parse;
mod prelude;
mod session;
mod span;
mod stmt;
mod token;
mod utils;
mod value;
mod warning;

pub use err::SodigyError;
pub use session::{GlobalParseSession, LocalParseSession};
pub use ast::AST;

use lexer::lex_tokens;
use stmt::parse_stmts;
use token::TokenList;

pub fn parse_file(s: &[u8], session: &mut LocalParseSession) -> Result<AST, Box<dyn SodigyError>> {
    let tokens = lex_tokens(s, session).map_err(|e| Box::new(e) as Box<dyn SodigyError>)?;
    let mut tokens = TokenList::from_vec(tokens);

    let stmts = parse_stmts(&mut tokens).map_err(|e| Box::new(e) as Box<dyn SodigyError>)?;

    AST::from_stmts(stmts, session).map_err(|e| Box::new(e) as Box<dyn SodigyError>)
}
