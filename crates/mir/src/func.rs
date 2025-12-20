use crate::{Expr, Session, Type};
use sodigy_hir::{self as hir, FuncParam, Generic};
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Func {
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<Generic>,
    pub params: Vec<FuncParam>,
    pub type_annotation_span: Option<Span>,
    pub value: Expr,
    pub built_in: bool,
}

impl Func {
    pub fn from_hir(hir_func: &hir::Func, session: &mut Session) -> Result<Func, ()> {
        let mut has_error = false;
        let mut params = Vec::with_capacity(hir_func.params.len());
        let mut param_types = Vec::with_capacity(hir_func.params.len());
        let type_annotation_span = hir_func.r#type.as_ref().map(|t| t.error_span_wide());

        for generic in hir_func.generics.iter() {
            session.generic_def_span_rev.insert(generic.name_span, hir_func.name_span);
        }

        for hir_param in hir_func.params.iter() {
            match hir_param.r#type.as_ref().map(|r#type| Type::from_hir(r#type, session)) {
                Some(Ok(r#type)) => {
                    param_types.push(r#type.clone());
                    session.types.insert(hir_param.name_span, r#type);
                },
                None => {
                    param_types.push(Type::Var {
                        def_span: hir_param.name_span,
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
                name_span: hir_param.name_span,
                r#type: None,
                default_value: hir_param.default_value,
            });
        }

        let value = match Expr::from_hir(&hir_func.value, session) {
            Ok(value) => Some(value),
            Err(()) => {
                has_error = true;
                None
            },
        };

        match hir_func.r#type.as_ref().map(|r#type| Type::from_hir(r#type, session)) {
            Some(Ok(r#type)) => {
                session.types.insert(
                    hir_func.name_span,
                    Type::Func {
                        // These spans are for type annotations, but there's no type annotation here!
                        fn_span: Span::None,
                        group_span: Span::None,
                        params: param_types,
                        r#return: Box::new(r#type),
                    },
                );
            },
            None => {
                session.types.insert(
                    hir_func.name_span,
                    Type::Func {
                        // These spans are for type annotations, but there's no type annotation here!
                        fn_span: Span::None,
                        group_span: Span::None,
                        params: param_types,
                        r#return: Box::new(Type::Var {
                            def_span: hir_func.name_span,
                            is_return: true,
                        }),
                    },
                );
            },
            Some(Err(())) => {
                has_error = true;
            },
        }

        if has_error {
            Err(())
        }

        else {
            Ok(Func {
                name: hir_func.name,
                name_span: hir_func.name_span,
                generics: hir_func.generics.to_vec(),
                params,
                type_annotation_span,
                value: value.unwrap(),
                built_in: hir_func.built_in,
            })
        }
    }
}
