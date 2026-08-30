use crate::{Expr, Session, Type};
use sodigy_error::FuncEffect;
use sodigy_hir::{self as hir, FuncOrigin, FuncParam, FuncShape, Generic};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

#[derive(Clone, Debug)]
pub struct Func {
    pub effect: FuncEffect,
    pub ndet_span: Option<Span>,
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<Generic>,
    pub generic_group_span: Option<Span>,
    pub params: Vec<FuncParam>,
    pub type_annot_span: Option<Span>,
    pub value: Expr,
    pub built_in: bool,
    pub origin: FuncOrigin,

    // #[unused_effect]
    pub unused_effect: bool,

    // Spans of `hir::Type::Wildcard`. It has to be monomorphized later.
    // Let's say there's an expression `foo.<_, Int>()`. The wildcard type
    // in the dotfish will be lowered to `Type::Var { def_span, .. }`, and
    // the def_span is the span of the wildcard token in the dotfish. When
    // the function is monomorphized, the span of the wild token also has
    // to be monomorphized!
    pub wildcard_spans: Vec<Span>,
}

impl Func {
    pub fn from_hir(hir_func: &hir::Func, session: &mut Session) -> Result<Func, ()> {
        session.wildcard_spans = vec![];

        let mut has_error = false;
        let mut params = Vec::with_capacity(hir_func.params.len());
        let mut param_types = Vec::with_capacity(hir_func.params.len());
        let type_annot_span = hir_func.type_annot.as_ref().map(|t| t.error_span_wide());
        let mut equal_generic_params: HashMap<Span, Vec<usize>> = HashMap::new();

        for (i, hir_param) in hir_func.params.iter().enumerate() {
            match hir_param.type_annot.as_ref().map(|type_annot| Type::from_hir(type_annot, session)) {
                Some(Ok(type_annot)) => {
                    if let Type::GenericParam { def_span, .. } = &type_annot {
                        match equal_generic_params.entry(def_span.clone()) {
                            Entry::Occupied(mut e) => {
                                e.get_mut().push(i);
                            },
                            Entry::Vacant(e) => {
                                e.insert(vec![i]);
                            },
                        }
                    }

                    param_types.push(type_annot.clone());
                    session.types.insert(hir_param.name_span.clone(), type_annot);
                },
                None => {
                    param_types.push(Type::Var {
                        def_span: hir_param.name_span.clone(),
                        is_return: false,
                    });
                },
                Some(Err(())) => {
                    has_error = true;
                    continue;
                },
            }

            params.push(FuncParam {
                name: hir_param.name,
                name_span: hir_param.name_span.clone(),
                type_annot: None,
                default_value: hir_param.default_value.clone(),
                unused_name: hir_param.unused_name.clone(),
            });
        }

        for indexes in equal_generic_params.values() {
            if indexes.len() > 1 {
                session.equal_generic_params.insert(
                    hir_func.name_span.clone(),
                    indexes[1..].iter().map(
                        |j| (indexes[0], *j)
                    ).collect(),
                );
            }
        }

        let value = match Expr::from_hir(&hir_func.value, session) {
            Ok(value) => Some(value),
            Err(()) => {
                has_error = true;
                None
            },
        };

        let return_type = match hir_func.type_annot.as_ref().map(|type_annot| Type::from_hir(type_annot, session)) {
            Some(Ok(return_type)) => return_type,
            None => Type::Var { def_span: hir_func.name_span.clone(), is_return: true },
            Some(Err(())) => {
                has_error = true;
                Type::Var { def_span: hir_func.name_span.clone(), is_return: true }
            },
        };

        session.types.insert(
            hir_func.name_span.clone(),
            Type::Func {
                // These spans are for `Fn` in type annotations, but there's no such thing here!
                fn_span: Span::None,
                group_span: Span::None,
                params: param_types,
                r#return: Box::new(return_type),
                effect: hir_func.effect.clone(),
            },
        );

        if has_error {
            Err(())
        }

        else {
            Ok(Func {
                effect: hir_func.effect.clone(),
                ndet_span: hir_func.ndet_span.clone(),
                keyword_span: hir_func.keyword_span.clone(),
                name: hir_func.name,
                name_span: hir_func.name_span.clone(),
                generics: hir_func.generics.to_vec(),
                generic_group_span: hir_func.generic_group_span.clone(),
                params,
                type_annot_span,
                value: value.unwrap(),
                built_in: hir_func.built_in,
                origin: hir_func.origin,
                unused_effect: hir_func.unused_effect,
                wildcard_spans: session.wildcard_spans.drain(..).collect(),
            })
        }
    }

    pub fn shape(&self) -> FuncShape {
        FuncShape {
            effect: self.effect.clone(),

            // type annotations are already erased
            params: self.params.clone(),
            generics: self.generics.clone(),
            generic_group_span: self.generic_group_span.clone(),
        }
    }
}
