use crate::{
    Expr,
    Func,
    Let,
    Namespace,
    NamespaceKind,
    Session,
};
use sodigy_parse as ast;

#[derive(Clone, Debug)]
pub struct Block {
    pub lets: Vec<Let>,
    pub funcs: Vec<Func>,

    // top-level block doesn't have a value
    pub value: Box<Option<Expr>>,
}

impl Block {
    pub fn from_ast(ast_block: &ast::Block, session: &mut Session) -> Result<Block, ()> {
        let mut lets = vec![];
        let mut funcs = vec![];
        let mut value = None;
        let mut has_error = false;

        session.name_stack.push(Namespace::new(NamespaceKind::Block, ast_block.iter_names().collect()));

        for r#let in ast_block.lets.iter() {
            match Let::from_ast(r#let, session) {
                Ok(l) => {
                    lets.push(l);
                },
                Err(_) => {
                    has_error = true;
                },
            }
        }

        for func in ast_block.funcs.iter() {
            match Func::from_ast(func, session) {
                Ok(f) => {
                    funcs.push(f);
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
                lets,
                funcs,
                value: Box::new(value),
            })
        }
    }
}
