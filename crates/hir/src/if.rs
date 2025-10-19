use crate::{Expr, FullPattern, Session};
use sodigy_parse as ast;
use sodigy_span::Span;

#[derive(Clone, Debug)]
pub struct If {
    pub if_span: Span,
    pub cond: Box<Expr>,
    pub pattern: Option<FullPattern>,
    pub else_span: Span,
    pub true_value: Box<Expr>,
    pub false_value: Box<Expr>,
}

impl If {
    pub fn from_ast(ast_if: &ast::If, session: &mut Session) -> Result<If, ()> {
        let (cond, true_value, false_value) = match (
            Expr::from_ast(ast_if.cond.as_ref(), session),
            Expr::from_ast(ast_if.true_value.as_ref(), session),
            Expr::from_ast(ast_if.false_value.as_ref(), session),
        ) {
            (Ok(cond), Ok(true_value), Ok(false_value)) => (cond, true_value, false_value),
            _ => {
                return Err(());
            },
        };

        let pattern = match &ast_if.pattern {
            Some(pattern) => match FullPattern::from_ast(pattern, session) {
                Ok(pattern) => Some(pattern),
                Err(_) => {
                    return Err(());
                },
            },
            None => None,
        };

        Ok(If {
            if_span: ast_if.if_span,
            cond: Box::new(cond),
            pattern,
            else_span: ast_if.else_span,
            true_value: Box::new(true_value),
            false_value: Box::new(false_value),
        })
    }
}
