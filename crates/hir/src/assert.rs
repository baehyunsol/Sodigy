use crate::{Expr, Session};
use sodigy_parse as ast;

#[derive(Clone, Debug)]
pub struct Assert {
    pub value: Expr,
}

impl Assert {
    pub fn from_ast(ast_assert: &ast::Assert, session: &mut Session) -> Result<Assert, ()> {
        Ok(Assert {
            value: Expr::from_ast(&ast_assert.value, session)?,
        })
    }
}
