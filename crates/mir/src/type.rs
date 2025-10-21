use crate::Session;
use sodigy_error::{Error, ErrorKind};
use sodigy_hir as hir;
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_span::Span;

#[derive(Clone, Debug)]
pub enum Type {
    // Int
    Static(Span /* def_span of `Int` */),

    // T in `fn first<T>(ls: [T]) -> T = ls[0];`
    GenericDef(Span /* def_span of `T` */),

    // ()
    Unit(Span /* group_span */),

    // Option<Int>, Option<T>, Option<Option<Int>>, ...
    // Tuple also has this type: `Generic { type: Unit, args: [..] }`
    Generic {
        r#type: Box<Type>,  // `Option`
        args: Vec<Type>,    // `[T]`
        group_span: Span,
    },

    // If a type annotation is missing, it creates a type variable.
    // The type variables will be infered.
    Var(Span /* def_span */),
}

impl Type {
    pub fn from_hir(hir_type: &hir::Type, session: &mut Session) -> Result<Type, ()> {
        match hir_type {
            hir::Type::Identifier(id) => match id.origin {
                NameOrigin::FuncArg { .. } => {
                    session.errors.push(Error {
                        kind: ErrorKind::DependentTypeNotAllowed,
                        span: id.span,
                        extra_span: Some(id.def_span),
                        ..Error::default()
                    });
                    Err(())
                },
                NameOrigin::Generic { .. } => Ok(Type::GenericDef(id.def_span)),
                NameOrigin::Local { kind } |
                NameOrigin::Foreign { kind } => match kind {
                    NameKind::Struct |
                    NameKind::Enum => Ok(Type::Static(id.def_span)),
                    _ => todo!(),
                },
            },
            hir::Type::Path { .. } => todo!(),
            hir::Type::Generic { r#type, args: hir_args, group_span } => {
                let mut has_error = false;
                let r#type = match Type::from_hir(r#type, session) {
                    Ok(r#type) => Some(r#type),
                    Err(()) => {
                        has_error = true;
                        None
                    },
                };
                let mut args = Vec::with_capacity(hir_args.len());

                for hir_arg in hir_args.iter() {
                    match Type::from_hir(hir_arg, session) {
                        Ok(arg) => {
                            args.push(arg);
                        },
                        Err(()) => {
                            has_error = true;
                        },
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(Type::Generic {
                        r#type: Box::new(r#type.unwrap()),
                        args,
                        group_span: *group_span,
                    })
                }
            },
            hir::Type::Tuple { types, group_span } => {
                if types.is_empty() {
                    Ok(Type::Unit(*group_span))
                } else {
                    let mut has_error = false;
                    let mut args = Vec::with_capacity(types.len());

                    for r#type in types.iter() {
                        match Type::from_hir(r#type, session) {
                            Ok(r#type) => {
                                args.push(r#type);
                            },
                            Err(()) => {
                                has_error = true;
                            },
                        }
                    }

                    if has_error {
                        Err(())
                    }

                    else {
                        Ok(Type::Generic {
                            r#type: Box::new(Type::Unit(*group_span)),
                            args,
                            group_span: *group_span,
                        })
                    }
                }
            },
            hir::Type::Func { .. } => todo!(),

            // it has to be infered
            hir::Type::Wildcard(span) => Ok(Type::Var(*span)),
        }
    }
}
