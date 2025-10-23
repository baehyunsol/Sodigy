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

    // Result<Int, String>, Result<T, U>, Option<Result<Int, String>>, ...
    // Tuple also has this type: `Param { type: Unit, args: [..] }`
    Param {
        r#type: Box<Type>,  // `Result`
        args: Vec<Type>,    // `[Int, String]`
        group_span: Span,
    },

    Func {
        fn_span: Span,
        args: Vec<Type>,
        r#return: Box<Type>,
    },

    // If a type annotation is missing, it creates a type variable.
    // The type variables will be infered.
    Var {
        // If a type annotation of a definition with `def_span` is missing,
        // its type is `Type::Var { def_span }`.
        def_span: Span,

        // If `is_return` is false, `types.get(def_span)` will give you `Type::Var { def_span }`
        // If `is_return` is true, `types.get(def_span)`
        //     will give you `Type::Func { args: [..], return: Type::Var { def_span } }`
        // You have to be careful when you update `types`.
        is_return: bool,
    },

    // It's also a type variable.
    //
    // ```
    // fn first<T>(ls: [T]) -> T = ls[0];
    // fn foo(ns: [Int]) = first(ns);
    // fn bar(ss: [String]) = first(ss);
    // ```
    //
    // `first` in `foo` has type `Fn([Int]) -> Int`, while
    // `first` in `bar` has type `Fn([String]) -> String`.
    // In order to infer this, we need a variable that represents
    // each instance of `T` in the invocations of `first`.
    GenericInstance {
        // span of `first` in `fn foo(ns) = first(ns);`
        call: Span,

        // span of `T` in `fn first<T>`
        generic: Span,
    },
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
            hir::Type::Param { r#type, args: hir_args, group_span } => {
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
                    Ok(Type::Param {
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
                        Ok(Type::Param {
                            r#type: Box::new(Type::Unit(*group_span)),
                            args,
                            group_span: *group_span,
                        })
                    }
                }
            },
            hir::Type::Func { .. } => todo!(),

            // it has to be infered
            hir::Type::Wildcard(span) => Ok(Type::Var {
                def_span: *span,
                is_return: false,
            }),
        }
    }

    pub fn get_type_vars(&self) -> Vec<Span> {
        match self {
            Type::Static(_) |
            Type::GenericDef(_) |
            Type::Unit(_) => vec![],
            Type::Param { r#type: t, args, .. } |
            Type::Func { r#return: t, args, .. } => {
                let mut result = t.get_type_vars();

                for arg in args.iter() {
                    result.extend(arg.get_type_vars());
                }

                result
            },
            Type::Var { def_span, .. } => vec![*def_span],
            Type::GenericInstance { .. } => todo!(),
        }
    }

    pub fn substitute(&mut self, type_var: Span, r#type: &Type) {
        match self {
            Type::Static(_) |
            Type::GenericDef(_) |
            Type::Unit(_) => {},
            Type::Param {
                r#type: t,
                args,
                ..
            } | Type::Func {
                r#return: t,
                args,
                ..
            } => {
                for arg in args.iter_mut() {
                    arg.substitute(type_var, r#type);
                }

                t.substitute(type_var, r#type);
            },
            Type::Var { def_span, .. } if *def_span == type_var => {
                *self = r#type.clone();
            },
            Type::Var { .. } => {},
            Type::GenericInstance { .. } => todo!(),
        }
    }
}
