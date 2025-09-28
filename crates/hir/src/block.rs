use crate::{
    Expr,
    Func,
    FuncOrigin,
    Let,
    Session,
};
use sodigy_name_analysis::{Namespace, NamespaceKind};
use sodigy_parse as ast;
use sodigy_span::Span;

#[derive(Clone, Debug)]
pub struct Block {
    pub group_span: Span,
    pub lets: Vec<Let>,
    pub value: Box<Option<Expr>>,
}

impl Block {
    pub fn from_ast(
        ast_block: &ast::Block,
        session: &mut Session,
        top_level: bool,
    ) -> Result<Block, ()> {
        let mut lets = vec![];
        let mut value = None;
        let mut has_error = false;

        session.name_stack.push(Namespace::new(NamespaceKind::Block, ast_block.iter_names().map(|(k, v1, v2)| (k, (v1, v2))).collect()));

        for r#let in ast_block.lets.iter() {
            match Let::from_ast(r#let, session, top_level) {
                Ok(l) => {
                    lets.push(l);
                },
                Err(_) => {
                    has_error = true;
                },
            }
        }

        let func_origin = if top_level {
            FuncOrigin::TopLevel
        } else {
            FuncOrigin::Inline
        };

        // All the function declarations are stored in the top-level block.
        for func in ast_block.funcs.iter() {
            match Func::from_ast(func, session, func_origin) {
                Ok(f) => {
                    session.funcs.push(f);
                },
                Err(_) => {
                    has_error = true;
                },
            }
        }

        if let Some(ast_value) = ast_block.value.as_ref() {
            match Expr::from_ast(ast_value, session) {
                Ok(v) => {
                    value = Some(v);
                },
                Err(_) => {
                    has_error = true;
                },
            }
        }

        session.name_stack.pop();

        if has_error {
            Err(())
        }

        else {
            Ok(Block {
                group_span: ast_block.group_span,
                lets,
                value: Box::new(value),
            })
        }
    }
}
