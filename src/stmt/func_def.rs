use super::{ArgDef, Decorator};
use crate::ast::{ASTError, NameScope};
use crate::expr::Expr;
use crate::session::InternedString;
use crate::span::Span;

pub struct FuncDef {
    pub span: Span,  // it points to `d` of `def`, or `\` of a lambda function
    pub name: InternedString,
    pub args: Vec<ArgDef>,

    pub decorators: Vec<Decorator>,

    // if it's None, it has to be inferred later (only lambda functions)
    pub ret_type: Option<Expr>,

    pub ret_val: Expr,

    // constants are defined without args 
    // 0-arg functions and constants are different: `def PI` vs `def GET_PI()`
    pub is_const: bool,
}

impl FuncDef {

    pub fn resolve_names(&mut self, name_scope: &mut NameScope) -> Result<(), ASTError> {

        for decorator in self.decorators.iter_mut() {
            decorator.resolve_names(name_scope)?;
        }

        name_scope.push_names(&self.args);

        // TODO: `push_names(self.args)` before this line? or after this?
        // dependent types?
        self.ret_val.resolve_names(name_scope)?;

        if let Some(ty) = &mut self.ret_type {
            ty.resolve_names(name_scope)?;
        }

        name_scope.pop_names();

        Ok(())
    }

    pub fn extract_lambdas(&mut self) -> Vec<FuncDef> {
        // TODO
        vec![]
    }

}