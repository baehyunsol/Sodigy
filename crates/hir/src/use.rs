use crate::Session;
use sodigy_parse as ast;

#[derive(Clone, Debug)]
pub struct Use {}

impl Use {
    pub fn from_ast(ast_use: &ast::Use, session: &mut Session) -> Result<Use, ()> {
        todo!()
    }
}
