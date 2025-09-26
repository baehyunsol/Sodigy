use sodigy_parse as ast;

// In sodigy_lex and sodigy_parse, the functions return `Result<T, Vec<Error>>`, and errors are handled by `?` operator.
// That means
// 1. The lexer and the parser doesn't generate warnings.
// 2. They usually halt immediately when there's a syntax error.
//
// But from hir, it has to generate warnings and continue after an error so that it can generate as many errors as possible.
// So all the errors and warnings are stored in the session, and the return value doesn't indicate anything about errors (it does, but don't rely on it).
// You first run the entire hir pass, then you have to check `session.errors` and `session.warnings`.

mod block;
mod expr;
mod func;
mod r#if;
mod r#let;
mod name;
mod pattern;
mod session;

pub use block::Block;
pub use expr::Expr;
pub use func::{CallArg, Func};
pub use r#if::If;
pub use r#let::Let;
pub use pattern::Pattern;
pub use session::Session;

pub(crate) use name::{NameOrigin, Namespace, NamespaceKind};

impl Session {
    /// Errors and warnings are stored in the session.
    pub fn lower(&mut self, ast_block: &ast::Block) -> Result<Block, ()> {
        Block::from_ast(ast_block, self)

        // TODO: find all lambda functions and convert them to normal functions
    }
}
