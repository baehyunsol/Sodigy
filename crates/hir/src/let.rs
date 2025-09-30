use crate::{Expr, Session};
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Let {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub r#type: Option<Expr>,
    pub value: Expr,
    pub origin: LetOrigin,

    // TODO: It has to track foreign names, like `Func`, so that we can draw a dependency graph between `let` values.
}

#[derive(Clone, Copy, Debug)]
pub enum LetOrigin {
    TopLevel,
    Inline,  // `let` keyword in an inline block
    FuncDefaultValue,
}

impl Let {
    pub fn from_ast(
        ast_let: &ast::Let,
        session: &mut Session,
        top_level: bool,
    ) -> Result<Let, ()> {
        let mut has_error = false;
        let mut r#type = None;

        if let Some(ast_type) = &ast_let.r#type {
            match Expr::from_ast(ast_type, session) {
                Ok(ty) => {
                    r#type = Some(ty);
                },
                Err(_) => {
                    has_error = true;
                },
            }
        }

        let value = match Expr::from_ast(&ast_let.value, session) {
            Ok(value) => Some(value),
            Err(_) => {
                has_error = true;
                None
            },
        };

        if has_error {
            Err(())
        }

        else {
            Ok(Let {
                keyword_span: ast_let.keyword_span,
                name: ast_let.name,
                name_span: ast_let.name_span,
                r#type,
                value: value.unwrap(),
                origin: if top_level {
                    LetOrigin::TopLevel
                } else {
                    LetOrigin::Inline
                },
            })
        }
    }
}
