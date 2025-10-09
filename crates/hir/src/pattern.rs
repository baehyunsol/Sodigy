use crate::Session;
use sodigy_parse as ast;

#[derive(Clone, Debug)]
pub struct Pattern;

impl Pattern {
    pub fn from_ast(ast_pattern: &ast::FullPattern, session: &mut Session) -> Result<Pattern, ()> {
        todo!()
    }
}
