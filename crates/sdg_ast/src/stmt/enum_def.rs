use super::{ArgDef, Decorator, FuncDef, FuncKind, GenericDef, VariantDef};
use crate::ast::NameOrigin;
use crate::err::ParamType;
use crate::expr::Expr;
use crate::path::Path;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::warning::SodigyWarning;
use sdg_uid::UID;
use std::collections::HashSet;

// it's later converted to multiple `FuncDef`s
pub struct EnumDef {
    def_span: Span,
    generics: Vec<GenericDef>,
    pub(crate) name_span: Span,
    pub(crate) name: InternedString,
    pub(crate) decorators: Vec<Decorator>,
    variants: Vec<VariantDef>,
    id: UID,
}

impl EnumDef {
    pub fn empty(def_span: Span, name_span: Span, name: InternedString, generics: Vec<GenericDef>) -> Self {
        EnumDef {
            def_span, name_span, name,
            decorators: vec![],
            variants: vec![],
            generics,
            id: UID::new_enum_id(),
        }
    }

    pub fn new(
        def_span: Span,
        name_span: Span,
        name: InternedString,
        variants: Vec<VariantDef>,
        generics: Vec<GenericDef>,
    ) -> Self {
        EnumDef {
            def_span, name_span, name,
            decorators: vec![],
            variants,
            generics,
            id: UID::new_enum_id(),
        }
    }

    /*
     * Enum Foo<T> { Var1, Var2(T), Var3(T, Int, Int) }
     *
     * # make sure that `Foo` and `Foo(Int)` are both valid
     * # let's say XXX is the id of `Foo`
     * # kind: Enum
     * def Foo<T>(T: Type): Type = Type.new(XXX);
     *
     * # add `Foo` to its path
     * # kind: EnumVariant
     * def Var1<T>: Foo(T) = Foo(T).variant(0);
     *
     * # add `Foo` to its path
     * # kind: EnumVariant
     * def Var2<T>(e0: T): Foo(T) = Foo(T).variant(1, (e0, ));
     *
     * # add `Foo` to its path
     * # kind: EnumVariant
     * def Var3<T>(e0: T, e1: Int, e2: Int): Foo(T) = Foo(T).variant(2, (e0, e1, e2));
     */
    pub fn to_defs(&self, location: &Path, session: &mut LocalParseSession) -> Vec<FuncDef> {
        let enum_def = FuncDef::enum_def(&self, location);
        let mut var_path = location.clone();
        var_path.push((enum_def.name, enum_def.name_span));

        let mut variants: Vec<FuncDef> = self.variants.iter().enumerate().map(
            |(index, variant)| FuncDef::enum_var(&self, variant, session, &var_path, index)
        ).collect();

        variants.push(enum_def);

        variants
    }

    // enum Foo<T> { A, B(C.T, D) }  -> `T` is unused, but can it catch that?
    pub fn check_unused_generics(&self, session: &mut LocalParseSession) {
        let mut used_names = HashSet::new();

        for var in self.variants.iter() {
            if let Some(fields) = &var.fields {
                for field in fields.iter() {
                    field.kind.id_walker(
                        &|&name, &origin, used_names: &mut HashSet<(InternedString, NameOrigin)>| {
                            used_names.insert((name, origin));
                        },
                        &mut used_names,
                    );
                }
            }
        }

        for GenericDef { name, span } in self.generics.iter() {
            if !used_names.contains(&(*name, NameOrigin::NotKnownYet)) {
                session.add_warning(SodigyWarning::unused_param(*name, *span, ParamType::FuncGeneric));
            }
        }
    }
}

impl FuncDef {
    pub fn enum_def(e: &EnumDef, location: &Path) -> FuncDef {
        FuncDef {
            def_span: e.def_span,
            name_span: e.name_span,
            name: e.name,
            args: e.generics.iter().map(|g| g.to_arg_def()).collect(),
            location: location.clone(),
            decorators: e.decorators.clone(),
            generics: e.generics.clone(),
            ret_type: Some(Expr::new_object(sdg_uid::prelude::type_(), Span::dummy())),
            ret_val: Expr::new_type_instance(e.id),
            kind: FuncKind::Enum,
            id: e.id,
        }
    }

    pub fn enum_var(
        parent: &EnumDef,
        variant: &VariantDef,
        session: &mut LocalParseSession,
        var_path: &Path,
        index: usize,
    ) -> FuncDef {
        let self_uid = UID::new_enum_var_id();
        let (args, kind, ret_val) = if let Some(fields) = &variant.fields {
            (
                fields.iter().enumerate().map(
                    |(index, ty)| ArgDef {
                        name: session.intern_string(format!("@@e{index}").as_bytes()),
                        ty: Some(ty.clone()),
                        span: ty.span,
                    }
                ).collect(),
                FuncKind::EnumVariantTuple(parent.id),
                Expr::new_enum_variant(parent.id, self_uid, index, fields, session),
            )
        } else {
            (
                vec![],
                FuncKind::EnumVariant(parent.id),
                Expr::new_enum_variant(parent.id, self_uid, index, &vec![], session),
            )
        };

        let ret_type = if parent.generics.is_empty() {
            Some(Expr::new_object(parent.id, Span::dummy()))
        } else {
            // Foo(T)
            Some(Expr::new_call(
                Expr::new_object(parent.id, Span::dummy()),
                parent.generics.iter().map(
                    |g| g.to_expr(self_uid)
                ).collect(),
                Span::dummy(),
            ))
        };

        FuncDef {
            def_span: Span::dummy(),
            name_span: variant.span,
            name: variant.name,
            args, kind,
            decorators: vec![],  // TODO: does it need decorators?
            generics: parent.generics.clone(),
            ret_type,
            ret_val,
            location: var_path.clone(),
            id: self_uid,
        }
    }
}
