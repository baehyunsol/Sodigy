use crate::Session;
use sodigy_parse as ast;

#[derive(Clone, Debug)]
pub struct Match {}

impl Match {
    pub fn from_ast(ast_match: &ast::Match, session: &mut Session) -> Result<Match, ()> {
        todo!()
    }
}
