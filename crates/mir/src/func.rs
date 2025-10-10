use crate::{Expr, Session, Type};
use sodigy_hir::{self as hir, FuncArgDef};
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Func {
    pub name: InternedString,
    pub name_span: Span,
    pub args: Vec<FuncArgDef<Type>>,
    pub r#type: Option<Type>,
    pub value: Expr,
}

impl Func {
    pub fn from_hir(hir_func: &hir::Func, session: &mut Session) -> Result<Func, ()> {
        let mut has_error = false;
        let mut args = Vec::with_capacity(hir_func.args.len());

        for hir_arg in hir_func.args.iter() {
            let r#type = match hir_arg.r#type.as_ref().map(|r#type| Type::from_hir(r#type, session)) {
                Some(Ok(r#type)) => Some(r#type),
                Some(Err(())) => {
                    has_error = true;
                    continue;
                },
                None => None,
            };

            args.push(FuncArgDef {
                name: hir_arg.name,
                name_span: hir_arg.name_span,
                r#type,
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

        let r#type = match hir_func.r#type.as_ref().map(|r#type| Type::from_hir(r#type, session)) {
            Some(Ok(r#type)) => Some(r#type),
            Some(Err(())) => {
                has_error = true;
                None
            },
            None => None,
        };

        if has_error {
            Err(())
        }

        else {
            Ok(Func {
                name: hir_func.name,
                name_span: hir_func.name_span,
                args,
                r#type,
                value: value.unwrap(),
            })
        }
    }
}
