use super::{ArgDef, Decorator};
use crate::ast::{ASTError, NameScope, NameScopeId, NameScopeKind};
use crate::expr::Expr;
use crate::hash::SdgHash;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use std::collections::HashMap;

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

    pub is_anonymous: bool,

    pub id: NameScopeId,
}

impl FuncDef {

    pub fn resolve_names(
        &mut self,
        name_scope: &mut NameScope,
        lambda_defs: &mut HashMap<InternedString, FuncDef>,
        session: &mut LocalParseSession,
    ) -> Result<(), ASTError> {

        for decorator in self.decorators.iter_mut() {
            decorator.resolve_names(name_scope, lambda_defs, session)?;
        }

        name_scope.push_names(&self.args, NameScopeKind::FuncArg(self.id));

        // TODO: `push_names(self.args)` before this line? or after this?
        // dependent types?
        self.ret_val.resolve_names(name_scope, lambda_defs, session)?;

        if let Some(ty) = &mut self.ret_type {
            ty.resolve_names(name_scope, lambda_defs, session)?;
        }

        for ArgDef { ty, .. } in self.args.iter_mut() {
            if let Some(ty) = ty {
                ty.resolve_names(name_scope, lambda_defs, session)?;
            }
        }

        name_scope.pop_names();

        Ok(())
    }

    pub fn create_anonymous_function(
        args: Vec<ArgDef>,
        ret_val: Expr,
        span: Span,
        id: NameScopeId,
        session: &mut LocalParseSession,
    ) -> Self {
        let lambda_func_name = format!("@@LAMBDA__{}", span.sdg_hash().to_string());

        FuncDef {
            args, ret_val, span, id,
            decorators: vec![],
            ret_type: None,  // has to be inferred later
            is_const: false,
            is_anonymous: true,
            name: session.intern_string(lambda_func_name.into()),
        }
    }

    pub fn is_closure(&self) -> bool {
        self.is_anonymous && todo!()
    }

}