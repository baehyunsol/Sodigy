use crate::Session;
use sodigy_parse as ast;

#[derive(Clone, Debug)]
pub enum MacroKind {}

impl MacroKind {
    pub fn from_ast(ast_macro: &ast::MacroKind, session: &mut Session) -> Result<MacroKind, ()> {
        todo!()
    }
}
