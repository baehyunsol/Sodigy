use crate::{Callable, Expr, Session};
use sodigy_error::{Error, ErrorKind};
use sodigy_hir::{self as hir, Generic, StructField};
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::unintern_string;
use std::collections::HashMap;

// This enum is originally meant for type annotations, but
// type-checker and type-inferer are also using this enum...
//
// `Eq` and `PartialEq` are only for type vars.
// For comparison, use `Solver::equal()` method.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    // Int
    Static {
        def_span: Span,
        span: Span,
    },

    // T in `fn first<T>(ls: [T]) -> T = ls[0];`
    GenericDef {
        def_span: Span,
        span: Span,
    },

    // ()
    Unit(Span /* group_span */),

    // !
    Never(Span),

    // Result<Int, String>, Result<T, U>, Option<Result<Int, String>>, ...
    // Tuple also has this type: `Param { type: Unit, args: [..] }`
    Param {
        r#type: Box<Type>,  // `Result`
        args: Vec<Type>,    // `<Int, String>`
        group_span: Span,
    },

    Func {
        fn_span: Span,
        group_span: Span,
        params: Vec<Type>,
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
        //     will give you `Type::Func { params: [..], return: Type::Var { def_span } }`
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
            hir::Type::Ident(id) => match id.origin {
                NameOrigin::FuncParam { .. } => {
                    let param_name = String::from_utf8_lossy(&unintern_string(id.id, &session.intermediate_dir).unwrap().unwrap()).to_string();
                    session.errors.push(Error {
                        kind: ErrorKind::DependentTypeNotAllowed,
                        spans: vec![
                            RenderableSpan {
                                span: id.span,
                                auxiliary: false,
                                note: Some(format!("The type annotation is using the name `{param_name}`, which is a function parameter.")),
                            },
                            RenderableSpan {
                                span: id.def_span,
                                auxiliary: true,
                                note: Some(format!("The parameter `{param_name}` is defined here.")),
                            },
                        ],
                        note: None,
                    });
                    Err(())
                },
                NameOrigin::Generic { .. } => Ok(Type::GenericDef {
                    def_span: id.def_span,
                    span: id.span,
                }),
                NameOrigin::Local { kind } |
                NameOrigin::Foreign { kind } => match kind {
                    NameKind::Struct |
                    NameKind::Enum => Ok(Type::Static {
                        def_span: id.def_span,
                        span: id.span,
                    }),
                    _ => {
                        session.errors.push(Error::todo(92226, &format!("lowering hir type: {hir_type:?}"), hir_type.error_span_wide()));
                        Err(())
                    },
                },
                NameOrigin::External => unreachable!(),
            },
            hir::Type::Path { .. } => {
                session.errors.push(Error::todo(33045, &format!("lowering hir type: {hir_type:?}"), hir_type.error_span_wide()));
                Err(())
            },
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
            hir::Type::Tuple { types: hir_types, group_span } => {
                if hir_types.is_empty() {
                    Ok(Type::Unit(*group_span))
                } else {
                    let mut has_error = false;
                    let mut types = Vec::with_capacity(hir_types.len());

                    for hir_type in hir_types.iter() {
                        match Type::from_hir(hir_type, session) {
                            Ok(r#type) => {
                                types.push(r#type);
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
                            args: types,
                            group_span: *group_span,
                        })
                    }
                }
            },
            hir::Type::Func { fn_span, group_span, params: hir_params, r#return } => {
                let mut has_error = false;
                let r#return = match Type::from_hir(r#return, session) {
                    Ok(r#type) => Some(r#type),
                    Err(()) => {
                        has_error = true;
                        None
                    },
                };
                let mut params = Vec::with_capacity(hir_params.len());

                for hir_param in hir_params.iter() {
                    match Type::from_hir(hir_param, session) {
                        Ok(param) => {
                            params.push(param);
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
                    Ok(Type::Func {
                        fn_span: *fn_span,
                        group_span: *group_span,
                        params,
                        r#return: Box::new(r#return.unwrap()),
                    })
                }
            },

            // it has to be infered
            hir::Type::Wildcard(span) => Ok(Type::Var {
                def_span: *span,
                is_return: false,
            }),
            hir::Type::Never(span) => Ok(Type::Never(*span)),
        }
    }

    pub fn get_type_vars(&self) -> Vec<Type> {
        match self {
            Type::Static { .. } |
            Type::GenericDef { .. } |
            Type::Unit(_) |
            Type::Never(_) => vec![],
            Type::Param { r#type: t, args, .. } |
            Type::Func { r#return: t, params: args, .. } => {
                let mut result = t.get_type_vars();

                for arg in args.iter() {
                    result.extend(arg.get_type_vars());
                }

                result
            },
            Type::Var { .. } | Type::GenericInstance { .. } => vec![self.clone()],
        }
    }

    pub fn substitute(&mut self, type_var: &Type, r#type: &Type) {
        match self {
            Type::Static { .. } |
            Type::GenericDef { .. } |
            Type::Unit(_) |
            Type::Never(_) => {},
            Type::Param {
                r#type: t,
                args,
                ..
            } | Type::Func {
                r#return: t,
                params: args,
                ..
            } => {
                for arg in args.iter_mut() {
                    arg.substitute(type_var, r#type);
                }

                t.substitute(type_var, r#type);
            },
            Type::Var { .. } | Type::GenericInstance { .. } => {
                if self == type_var {
                    *self = r#type.clone();
                }
            },
        }
    }

    pub fn substitute_generic_def(&mut self, call: Span, generics: &[Span]) {
        match self {
            Type::GenericDef { def_span, .. } => {
                if generics.contains(def_span) {
                    *self = Type::GenericInstance { call, generic: *def_span };
                }
            },
            Type::Static { .. } |
            Type::Unit(_) |
            Type::Never(_) |
            Type::Var { .. } |
            Type::GenericInstance { .. } => {},
            Type::Param {
                r#type: t,
                args,
                ..
            } | Type::Func {
                r#return: t,
                params: args,
                ..
            } => {
                for arg in args.iter_mut() {
                    arg.substitute_generic_def(call, generics);
                }

                t.substitute_generic_def(call, generics);
            },
        }
    }

    pub fn generic_to_type_var(&mut self) {
        match self {
            Type::GenericDef { def_span, .. } => {
                *self = Type::Var { def_span: *def_span, is_return: false };
            },
            Type::Static { .. } |
            Type::Unit(_) |
            Type::Never(_) |
            Type::Var { .. } |
            Type::GenericInstance { .. } => {},
            Type::Param {
                r#type: t,
                args,
                ..
            } | Type::Func {
                r#return: t,
                params: args,
                ..
            } => {
                for arg in args.iter_mut() {
                    arg.generic_to_type_var();
                }

                t.generic_to_type_var();
            },
        }
    }
}

/// It returns the type of `expr`, assuming that type-check and type-infer are
/// complete. If type-check or type-infer is incomplete, it'll return None,
/// or even worse, a wrong type.
pub fn type_of(
    expr: &Expr,
    types: &HashMap<Span, Type>,
    struct_shapes: &HashMap<Span, (Vec<StructField>, Vec<Generic>)>,
    lang_items: &HashMap<String, Span>,
) -> Option<Type> {
    match expr {
        Expr::Ident(id) => types.get(&id.def_span).map(|r#type| r#type.clone()),
        Expr::Number { n, .. } => match n.is_integer {
            true => Some(Type::Static {
                def_span: *lang_items.get("type.Int").unwrap(),
                span: Span::None,
            }),
            false => Some(Type::Static {
                def_span: *lang_items.get("type.Number").unwrap(),
                span: Span::None,
            }),
        },
        Expr::String { binary, .. } => match *binary {
            true => Some(Type::Param {
                r#type: Box::new(Type::Static {
                    def_span: *lang_items.get("type.List").unwrap(),
                    span: Span::None,
                }),
                args: vec![Type::Static {
                    def_span: *lang_items.get("type.Byte").unwrap(),
                    span: Span::None,
                }],
                group_span: Span::None,
            }),
            false => Some(Type::Param {
                r#type: Box::new(Type::Static {
                    def_span: *lang_items.get("type.List").unwrap(),
                    span: Span::None,
                }),
                args: vec![Type::Static {
                    def_span: *lang_items.get("type.Char").unwrap(),
                    span: Span::None,
                }],
                group_span: Span::None,
            }),
        },
        Expr::Char { .. } => Some(Type::Static {
            def_span: *lang_items.get("type.Char").unwrap(),
            span: Span::None,
        }),
        Expr::Byte { .. } => Some(Type::Static {
            def_span: *lang_items.get("type.Byte").unwrap(),
            span: Span::None,
        }),
        Expr::If(r#if) => type_of(&r#if.true_value, types, struct_shapes, lang_items),
        Expr::Match(r#match) => type_of(&r#match.arms[0].value, types, struct_shapes, lang_items),
        Expr::MatchFsm(match_fsm) => todo!(),
        Expr::Block(block) => type_of(&block.value, types, struct_shapes, lang_items),
        Expr::Call { func, args, .. } => match func {
            Callable::Static { def_span, .. } => match types.get(def_span) {
                Some(Type::Func { r#return, .. }) => Some(*r#return.clone()),
                _ => None,
            },
            Callable::StructInit { def_span, .. } => Some(Type::Static {
                def_span: *def_span,
                span: Span::None,
            }),
            Callable::TupleInit { .. } => {
                let mut arg_types = Vec::with_capacity(args.len());

                for arg in args.iter() {
                    match type_of(arg, types, struct_shapes, lang_items) {
                        Some(t) => { arg_types.push(t); },
                        None => { return None; },
                    }
                }

                Some(Type::Param {
                    // `Type::Unit`'s `group_span` is of type annotation,
                    // and `Callable::TupleInit`'s `group_span` is of the expression.
                    r#type: Box::new(Type::Unit(Span::None)),
                    args: arg_types,

                    // this is for the type annotation, hence None
                    group_span: Span::None,
                })
            },
            _ => todo!(),
        },
        _ => todo!(),
    }
}
