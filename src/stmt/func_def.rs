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
        name_scope.name_stack.push(
            self.args.iter().map(
                |arg| arg.name
            ).collect()
        );

        self.ret_val.resolve_names(name_scope)
    }

}