use crate::Session;
use sodigy_parse as ast;

// TODO: more fine-grained publicity
#[derive(Clone, Debug)]
pub struct Public(pub bool);

impl Public {
    // TODO: more fine-grained publicity
    pub fn from_ast(ast_public: &Option<ast::Public>, session: &mut Session) -> Result<Public, ()> {
        Ok(Public(ast_public.is_some()))
    }

    // TODO: more fine-grained publicity
    pub fn private() -> Self {
        Public(false)
    }

    // TODO: more fine-grained publicity
    pub fn is_public(&self) -> bool {
        self.0
    }
}
