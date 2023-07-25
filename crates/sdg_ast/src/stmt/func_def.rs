use super::{ArgDef, Decorator};
use crate::ast::{ASTError, NameOrigin, NameScope, NameScopeKind};
use crate::err::ParamType;
use crate::expr::Expr;
use crate::module::ModulePath;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::warning::SodigyWarning;
use sdg_hash::SdgHash;
use sdg_uid::UID;
use std::collections::{HashMap, HashSet};

pub const LAMBDA_FUNC_PREFIX: &str = "@@LAMBDA__";

pub struct FuncDef {
    pub span: Span,  // it points to `d` of `def`, or `\` of a lambda function
    pub name: InternedString,
    pub args: Vec<ArgDef>,

    pub location: ModulePath,

    pub decorators: Vec<Decorator>,

    // if it's None, it has to be inferred later (only lambda functions)
    pub ret_type: Option<Expr>,

    pub ret_val: Expr,

    pub(crate) kind: FuncKind,

    pub id: UID,
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
        let kind = if is_const {
            FuncKind::Const
        } else {
            FuncKind::Normal
        };

        FuncDef {
            name, args,
            ret_type: Some(ret_type),
            ret_val,
            span,
            location: ModulePath::empty(),  // will be filled later
            kind,
            decorators: vec![],  // filled later
            id: UID::new_func_id(),
        }
    }

    pub fn create_anonymous_function(
        args: Vec<ArgDef>,
        ret_val: Expr,
        span: Span,
        id: UID,
        session: &mut LocalParseSession,
    ) -> Self {
        let lambda_func_name = format!("{LAMBDA_FUNC_PREFIX}{}", span.sdg_hash().to_string());

        FuncDef {
            args, ret_val, span, id,
            location: ModulePath::empty(),  // nobody cares!
            decorators: vec![],
            ret_type: None,  // has to be inferred later
            kind: FuncKind::Lambda,  // if it's a closure, it'll be handled later
            name: session.intern_string(lambda_func_name.into()),
        }
    }

    fn is_anonymous(&self) -> bool {
        match self.kind {
            FuncKind::Closure | FuncKind::Lambda => true,
            FuncKind::Normal | FuncKind::Const
            | FuncKind::Enum | FuncKind::Struct
            | FuncKind::EnumVariant => false,
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

        session.add_warnings(self.get_unused_name_warnings(&used_args));

        name_scope.pop_names();

        Ok(())
    }

    // It returns Some(_) only when the result is non-empty
    // That's for easier pattern destructuring
    pub fn get_all_foreign_names(&self) -> Option<HashSet<(InternedString, NameOrigin)>> {
        if !self.is_anonymous() {
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

    pub fn get_unused_name_warnings(&self, used_names: &HashSet<(InternedString, NameOrigin)>) -> Vec<SodigyWarning> {
        let mut result = vec![];
        let self_name_origin = NameOrigin::FuncArg(self.id);

        let param_type = if self.is_anonymous() {
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

    pub fn dump(&self, session: &LocalParseSession) -> String {
        format!(
            "#kind: {:?}{}\ndef {}({}): {} = {};",
            self.kind,
            self.decorators.iter().map(
                |deco| format!("\n{}", deco.to_string(session))
            ).collect::<Vec<String>>().concat(),
            self.name.to_string(session),
            self.args.iter().map(
                |arg| arg.to_string(session)
            ).collect::<Vec<String>>().join(", "),
            if let Some(ty) = &self.ret_type {
                ty.to_string(session)
            } else {
                String::from("@DontKnow")
            },
            self.ret_val.to_string(session),
        )
    }
}

#[derive(Debug)]
pub enum FuncKind {

    // def foo(n: Int): Int = n + 1;
    Normal,

    // def PI: Number = 3.14159;
    Const,

    // \{x, y, x + y}
    Lambda,

    // \{x, x + n}
    Closure,

    Enum,
    EnumVariant,
    Struct,
}
