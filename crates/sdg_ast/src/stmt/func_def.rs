use super::{ArgDef, Decorator};
use crate::ast::{ASTError, NameOrigin, NameScope, NameScopeId, NameScopeKind};
use crate::err::ParamType;
use crate::expr::Expr;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::warning::SodigyWarning;
use sdg_hash::SdgHash;
use std::collections::{HashMap, HashSet};

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
    is_const: bool,

    is_anonymous: bool,

    pub id: NameScopeId,
}

impl FuncDef {

    pub fn new(
        name: InternedString,
        args: Vec<ArgDef>,
        is_const: bool,
        ret_type: Expr,
        ret_val: Expr,
        span: Span,
    ) -> Self {
        FuncDef {
            name, args, is_const,
            ret_type: Some(ret_type),
            ret_val,
            span,
            is_anonymous: false,
            decorators: vec![],  // filled later
            id: NameScopeId::new_rand(),
        }
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

    pub fn resolve_names(
        &mut self,
        name_scope: &mut NameScope,
        lambda_defs: &mut HashMap<InternedString, FuncDef>,
        session: &mut LocalParseSession,
    ) -> Result<(), ASTError> {

        // it's used to emit `warning: unused arg ...`
        let mut used_args = HashSet::new();

        for decorator in self.decorators.iter_mut() {
            decorator.resolve_names(name_scope, lambda_defs, session)?;
        }

        name_scope.push_names(&self.args, NameScopeKind::FuncArg(self.id));

        // TODO: `push_names(self.args)` before this line? or after this?
        // dependent types?
        self.ret_val.resolve_names(name_scope, lambda_defs, session, &mut used_args)?;

        if let Some(ty) = &mut self.ret_type {
            ty.resolve_names(name_scope, lambda_defs, session, &mut used_args)?;
        }

        for ArgDef { ty, .. } in self.args.iter_mut() {
            if let Some(ty) = ty {
                ty.resolve_names(name_scope, lambda_defs, session, &mut used_args)?;
            }
        }

        session.add_warnings(self.get_unused_names(&used_args));

        name_scope.pop_names();

        Ok(())
    }

    // It returns Some(_) only when the result is non-empty
    // That's for easier pattern destructuring
    pub fn get_all_foreign_names(&self) -> Option<HashSet<(InternedString, NameOrigin)>> {
        if !self.is_anonymous {
            None
        } else {
            let mut result = HashSet::new();
            let mut blocks = vec![];
            self.ret_val.get_all_foreign_names(self.id, &mut result, &mut blocks);

            for ArgDef { ty, .. } in self.args.iter() {
                if let Some(ty) = ty {
                    ty.get_all_foreign_names(self.id, &mut result, &mut blocks);
                }
            }

            if result.is_empty() {
                None
            }

            else {
                Some(result)
            }
        }
    }

    pub fn get_unused_names(&self, used_names: &HashSet<(InternedString, NameOrigin)>) -> Vec<SodigyWarning> {
        let mut result = vec![];
        let self_name_origin = NameOrigin::FuncArg(self.id);

        let param_type = if self.is_anonymous {
            ParamType::LambdaParam
        } else {
            ParamType::FuncParam
        };

        for ArgDef { name, span, .. } in self.args.iter() {
            if !used_names.contains(&(*name, self_name_origin)) {
                result.push(SodigyWarning::unused(*name, *span, param_type));
            }
        }

        result
    }
}