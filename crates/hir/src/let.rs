use crate::{Expr, Session};
use sodigy_parse as ast;

#[derive(Clone, Debug)]
pub struct Let {
    value: Expr,
}

impl Let {
    pub fn from_ast(ast_let: &ast::Let, session: &mut Session) -> Result<Let, ()> {
        let value = Expr::from_ast(&ast_let.value, session)?;

        Ok(Let {
            value,
        })
    }
}
