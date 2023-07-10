use super::{ArgDef, Decorator};
use crate::ast::{ASTError, NameScope};
use crate::expr::Expr;
use crate::session::InternedString;
use crate::span::Span;

pub struct FuncDef {
    pub span: Span,  // it points to `d` of `def`
    pub name: InternedString,
    pub args: Vec<ArgDef>,

    pub decorators: Vec<Decorator>,

    pub ret_type: Expr,
    pub ret_val: Expr,

    // constants are defined without args 
    // 0-arg functions and constants are different: `def PI` vs `def GET_PI()`
    pub is_const: bool,
}

impl FuncDef {

    pub fn resolve_names(&mut self, name_scope: &mut NameScope) -> Result<(), ASTError> {
        name_scope.push_names(&self.args);

        // TODO: `push_names(self.args)` before this line? or after this?
        // dependent types?
        self.ret_type.resolve_names(name_scope)?;
        self.ret_val.resolve_names(name_scope)?;

        name_scope.pop_names();

        Ok(())
    }

}