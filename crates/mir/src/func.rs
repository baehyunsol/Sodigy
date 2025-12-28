use crate::{Expr, Session, Type};
use sodigy_hir::{self as hir, FuncParam, FuncPurity, Generic};
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Func {
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<Generic>,
    pub params: Vec<FuncParam>,
    pub type_annot_span: Option<Span>,
    pub value: Expr,
    pub built_in: bool,
}

impl Func {
    pub fn from_hir(hir_func: &hir::Func, session: &mut Session) -> Result<Func, ()> {
        let mut has_error = false;
        let mut params = Vec::with_capacity(hir_func.params.len());
        let mut param_types = Vec::with_capacity(hir_func.params.len());
        let type_annot_span = hir_func.type_annot.as_ref().map(|t| t.error_span_wide());

        for generic in hir_func.generics.iter() {
            session.generic_def_span_rev.insert(generic.name_span, hir_func.name_span);
        }

        for hir_param in hir_func.params.iter() {
            match hir_param.type_annot.as_ref().map(|type_annot| Type::from_hir(type_annot, session)) {
                Some(Ok(type_annot)) => {
                    param_types.push(type_annot.clone());
                    session.types.insert(hir_param.name_span, type_annot);
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
                type_annot: None,
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

        match hir_func.type_annot.as_ref().map(|type_annot| Type::from_hir(type_annot, session)) {
            Some(Ok(type_annot)) => {
                session.types.insert(
                    hir_func.name_span,
                    Type::Func {
                        // These spans are for `Fn` in type annotations, but there's no such thing here!
                        fn_span: Span::None,
                        group_span: Span::None,
                        params: param_types,
                        r#return: Box::new(type_annot),
                        purity: if hir_func.is_pure { FuncPurity::Pure } else { FuncPurity::Impure },
                    },
                );
            },
            None => {
                session.types.insert(
                    hir_func.name_span,
                    Type::Func {
                        // These spans are for `Fn` in type annotations, but there's no such thing here!
                        fn_span: Span::None,
                        group_span: Span::None,
                        params: param_types,
                        r#return: Box::new(Type::Var {
                            def_span: hir_func.name_span,
                            is_return: true,
                        }),
                        purity: if hir_func.is_pure { FuncPurity::Pure } else { FuncPurity::Impure },
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
                type_annot_span,
                value: value.unwrap(),
                built_in: hir_func.built_in,
            })
        }
    }
}
