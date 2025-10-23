use crate::{Expr, Session, Type};
use sodigy_hir::{self as hir, FuncArgDef};
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Func {
    pub name: InternedString,
    pub name_span: Span,
    pub args: Vec<FuncArgDef<()>>,
    pub type_annotation_span: Option<Span>,
    pub value: Expr,
}

impl Func {
    pub fn from_hir(hir_func: &hir::Func, session: &mut Session) -> Result<Func, ()> {
        let mut has_error = false;
        let mut args = Vec::with_capacity(hir_func.args.len());
        let mut arg_types = Vec::with_capacity(hir_func.args.len());
        let type_annotation_span = hir_func.r#type.as_ref().map(|t| t.error_span());

        for hir_arg in hir_func.args.iter() {
            match hir_arg.r#type.as_ref().map(|r#type| Type::from_hir(r#type, session)) {
                Some(Ok(r#type)) => {
                    arg_types.push(r#type.clone());
                    session.types.insert(hir_arg.name_span, r#type);
                },
                None => {
                    arg_types.push(Type::Var {
                        def_span: hir_arg.name_span,
                        is_return: false,
                    });
                },
                Some(Err(())) => {
                    has_error = true;
                    continue;
                },
            }

            args.push(FuncArgDef {
                name: hir_arg.name,
                name_span: hir_arg.name_span,
                r#type: None,
                default_value: hir_arg.default_value,
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
                        // This is for `Fn` identifier in the type annotation, not the `fn` keyword!
                        fn_span: Span::None,
                        args: arg_types,
                        r#return: Box::new(r#type),
                    },
                );
            },
            None => {
                session.types.insert(
                    hir_func.name_span,
                    Type::Func {
                        // This is for `Fn` identifier in the type annotation, not the `fn` keyword!
                        fn_span: Span::None,
                        args: arg_types,
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
                args,
                type_annotation_span,
                value: value.unwrap(),
            })
        }
    }
}
