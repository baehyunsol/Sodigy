use crate::{Func, Let, Namespace, NamespaceKind, Session};
use sodigy_parse as ast;

#[derive(Clone, Debug)]
pub struct Block {
    funcs: Vec<Func>,
    lets: Vec<Let>,
}

impl Block {
    pub fn from_ast(ast_block: &ast::Block, session: &mut Session) -> Result<Block, ()> {
        let mut funcs = vec![];
        let mut lets = vec![];
        let mut has_error = false;

        session.name_stack.push(Namespace::new(NamespaceKind::Block, ast_block.iter_names().collect()));

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

        session.name_stack.pop();

        if has_error {
            Err(())
        }

        else {
            Ok(Block {
                funcs,
                lets,
            })
        }
    }
}
