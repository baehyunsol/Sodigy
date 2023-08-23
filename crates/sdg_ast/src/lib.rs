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

pub use ast::AST;
pub use err::SodigyError;
pub use expr::{Expr, ExprKind};
pub use session::{GlobalParseSession, InternedString, LocalParseSession};
pub use stmt::{FuncDef, FuncKind};
pub use value::ValueKind;

use err::ParseError;
use lexer::lex_tokens;
use span::Span;
use stmt::parse_stmts;
use token::TokenList;

/// If it returns `Err(())`, the actual errors are in `session`.
/// You have to set the input of `session` before calling this function.
/// In most cases, it's way more convenient to call `parse_files` rather than this function.
pub fn parse_file(s: &[u8], session: &mut LocalParseSession) -> Result<AST, ()> {
    let tokens = lex_tokens(s, session).map_err(|e| session.try_add_error::<(), ParseError>(Err(e)))?;
    let mut tokens = TokenList::from_vec(tokens, Span::new(session.curr_file, 0, 0));
    let stmts = parse_stmts(&mut tokens, session)?;

    AST::from_stmts(stmts, session)
}

pub fn parse_files(path: String, session: &mut LocalParseSession) -> Result<Vec<AST>, ()> {
    let mut paths_to_check = vec![(path, Span::dummy())];
    let mut asts = vec![];

    while let Some((path, span)) = paths_to_check.pop() {
        if let Err(mut e) = session.set_input(&path) {
            e.set_span(vec![span]);
            session.add_error(e);
            continue;
        }

        let input = session.get_curr_file_content().to_vec();

        // it continues parsing files even though a file has an error.
        // so that it can find as many errors as possible
        if let Ok(ast) = parse_file(&input, session) {
            for path_and_span in ast.get_path_of_inner_modules(session).into_iter() {
                paths_to_check.push(path_and_span);
            }

            asts.push(ast);
        }

    }

    session.err_if_has_error()?;

    Ok(asts)
}
