use crate::Session;
use sodigy_parse as ast;

#[derive(Clone, Debug)]
pub struct Type;

impl Type {
    pub fn from_ast(ast_struct: &ast::Type, session: &mut Session) -> Result<Type, ()> {
        todo!()
    }
}
