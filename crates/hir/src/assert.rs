use crate::{Expr, Session};
use sodigy_parse as ast;
use sodigy_span::Span;

#[derive(Clone, Debug)]
pub struct Assert {
    pub keyword_span: Span,
    pub value: Expr,
}

impl Assert {
    pub fn from_ast(ast_assert: &ast::Assert, session: &mut Session) -> Result<Assert, ()> {
        Ok(Assert {
            keyword_span: ast_assert.keyword_span,
            value: Expr::from_ast(&ast_assert.value, session)?,
        })
    }
}
