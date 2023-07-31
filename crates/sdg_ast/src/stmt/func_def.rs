use super::{ArgDef, Decorator, GenericDef};
use crate::ast::{NameOrigin, NameScope, NameScopeKind};
use crate::err::ParamType;
use crate::expr::Expr;
use crate::path::Path;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::warning::SodigyWarning;
use sdg_hash::SdgHash;
use sdg_uid::UID;
use std::collections::{HashMap, HashSet};

#[cfg(test)]
use crate::utils::assert_identifier;

pub const LAMBDA_FUNC_PREFIX: &str = "@@LAMBDA__";

pub struct FuncDef {
    pub def_span: Span,  // keyword `def` or `\` in lambda
    pub name_span: Span,
    pub name: InternedString,
    pub args: Vec<ArgDef>,

    pub location: Path,

    pub decorators: Vec<Decorator>,

    // TODO: is it a constant if it has generic params?
    pub generics: Vec<GenericDef>,

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
        generics: Vec<GenericDef>,
        def_span: Span,
        name_span: Span,
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
            def_span,
            name_span,
            generics,
            location: Path::empty(),  // will be filled later
            kind,
            decorators: vec![],  // filled later
            id: UID::new_func_id(),
        }
    }

    pub fn create_anonymous_function(
        args: Vec<ArgDef>,
        ret_val: Expr,
        def_span: Span,
        id: UID,
        session: &mut LocalParseSession,
    ) -> Self {
        let lambda_func_name = format!(
            "{LAMBDA_FUNC_PREFIX}{}",
            String::from_utf8_lossy(&def_span.sdg_hash().to_bytes()[..24]),
        );

        FuncDef {
            args, ret_val, id,
            def_span,
            name_span: Span::dummy(),
            location: Path::empty(),  // nobody cares!
            decorators: vec![],
            generics: vec![],
            ret_type: None,  // has to be inferred later
            kind: FuncKind::Lambda,  // if it's a closure, it'll be handled later
            name: session.intern_string(lambda_func_name.into()),
        }
    }

    fn is_anonymous(&self) -> bool {
        match self.kind {
            FuncKind::Closure(_) | FuncKind::Lambda => true,
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
    ) {

        // it's used to emit `warning: unused arg ...`
        let mut used_args = HashSet::new();

        for decorator in self.decorators.iter_mut() {
            decorator.resolve_names(name_scope, lambda_defs, session);
        }

        name_scope.push_names(&self.generics, NameScopeKind::GenericArg(self.id));
        name_scope.push_names(&self.args, NameScopeKind::FuncArg(self.id));

        // For now, types are dependent to the args
        for ArgDef { ty, .. } in self.args.iter_mut() {
            if let Some(ty) = ty {
                ty.resolve_names(name_scope, lambda_defs, session, &mut used_args);
            }
        }

        if let Some(ty) = &mut self.ret_type {
            ty.resolve_names(name_scope, lambda_defs, session, &mut used_args);
        }

        self.ret_val.resolve_names(name_scope, lambda_defs, session, &mut used_args);

        session.add_warnings(self.get_unused_name_warnings(&used_args));

        // one for args, one for generics
        name_scope.pop_names();
        name_scope.pop_names();
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

            if let Some(ty) = &self.ret_type {
                ty.get_all_foreign_names(self.id, &mut result, &mut blocks);
            }

            if result.is_empty() {
                None
            }

            else {
                Some(result)
            }
        }
    }

    pub fn get_unused_name_warnings(
        &self,
        used_names: &HashSet<(InternedString, NameOrigin)>
    ) -> Vec<SodigyWarning> {
        let mut warnings = vec![];
        let func_name_origin = NameOrigin::FuncArg(self.id);
        let generic_name_origin = NameOrigin::GenericArg(self.id);

        let param_type = if self.is_anonymous() {
            ParamType::LambdaParam
        } else {
            ParamType::FuncParam
        };

        for ArgDef { name, span, .. } in self.args.iter() {
            if !used_names.contains(&(*name, func_name_origin)) {
                warnings.push(SodigyWarning::unused(*name, *span, param_type));
            }
        }

        for GenericDef { name, span } in self.generics.iter() {
            if !used_names.contains(&(*name, generic_name_origin)) {
                warnings.push(SodigyWarning::unused(*name, *span, ParamType::FuncGeneric));
            }
        }

        warnings
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        #[cfg(test)]
        {
            let def = self.def_span.dump(session);

            if def != "def" && def != "\\" {
                panic!("{def}");
            }

            assert_identifier(self.name_span.dump(session));
        }

        format!(
            "#kind: {}{}\ndef {}{}({}): {} = {};",
            self.kind.to_string(session),
            self.decorators.iter().map(
                |deco| format!("\n{}", deco.dump(session))
            ).collect::<Vec<String>>().concat(),
            self.name.to_string(session),
            if self.generics.is_empty() {
                String::new()
            } else {
                format!(
                    "<{}>",
                    self.generics.iter().map(
                        |gen| gen.dump(session)
                    ).collect::<Vec<String>>().join(", ")
                )
            },
            self.args.iter().map(
                |arg| arg.dump(session)
            ).collect::<Vec<String>>().join(", "),
            if let Some(ty) = &self.ret_type {
                ty.dump(session)
            } else {
                String::from("@DontKnow")
            },
            self.ret_val.dump(session),
        )
    }
}

pub enum FuncKind {

    // def foo(n: Int): Int = n + 1;
    Normal,

    // def PI: Number = 3.14159;
    Const,

    // \{x, y, x + y}
    Lambda,

    // \{x, x + n}
    // the associated data is captured variables
    Closure(Vec<(InternedString, NameOrigin)>),

    Enum,
    EnumVariant,
    Struct,
}

impl FuncKind {
    pub fn to_string(&self, session: &LocalParseSession) -> String {
        match self {
            FuncKind::Normal => "normal".to_string(),
            FuncKind::Const => "const".to_string(),
            FuncKind::Lambda => "lambda".to_string(),
            FuncKind::Enum => "enum".to_string(),
            FuncKind::EnumVariant => "enum variant".to_string(),
            FuncKind::Struct => "struct".to_string(),
            FuncKind::Closure(captured_variables) => format!(
                "closure({})",
                captured_variables.iter().map(
                    |(var, _)| var.to_string(session)
                ).collect::<Vec<String>>().join(", ")
            ),
        }
    }
}
