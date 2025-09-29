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
mod r#enum;
mod expr;
mod func;
mod r#if;
mod r#let;
mod name;
mod pattern;
mod session;
mod r#struct;

pub use block::Block;
pub use r#enum::Enum;
pub use expr::Expr;
pub use func::{CallArg, Func, FuncArgDef, FuncOrigin};
pub use r#if::If;
pub use r#let::{Let, LetOrigin};
pub use pattern::Pattern;
pub use session::Session;
pub use r#struct::{Struct, StructInitField};

impl Session {
    /// Errors and warnings are stored in the session.
    pub fn lower(&mut self, ast_block: &ast::Block) -> Result<(), ()> {
        let mut top_level_block = Block::from_ast(ast_block, self, true /* is_top_level */)?;

        for r#let in top_level_block.lets.drain(..) {
            self.lets.push(r#let);
        }

        Ok(())
    }
}
