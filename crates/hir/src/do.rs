use crate::{Expr, Session};
use sodigy_parse as ast;
use sodigy_span::Span;

#[derive(Clone, Debug)]
pub struct Do {
    pub keyword_span: Span,
    pub value: Expr,
}

impl Do {
    pub fn from_ast(ast_do: &ast::Do, session: &mut Session) -> Result<Do, ()> {
        Ok(Do {
            keyword_span: ast_do.keyword_span.clone(),
            value: Expr::from_ast(&ast_do.value, session)?,
        })
    }
}
