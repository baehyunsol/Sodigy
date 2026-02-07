use crate::{Callable, Expr, Session};
use sodigy_error::{Error, ErrorKind};
use sodigy_hir::{self as hir, FuncPurity, StructShape};
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_span::Span;
use sodigy_token::Constant;
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
    GenericParam {
        def_span: Span,
        span: Span,
    },

    // !
    Never(Span),

    Tuple {
        args: Vec<Type>,

        // of type annotation
        group_span: Span,
    },

    // Result<Int, String>
    Param {
        constructor_def_span: Span,  // of `Result`
        constructor_span: Span,  // of `Result` in the type annotation
        args: Vec<Type>,    // `<Int, String>`
        group_span: Span,
    },

    Func {
        fn_span: Span,
        group_span: Span,
        params: Vec<Type>,
        r#return: Box<Type>,
        purity: FuncPurity,
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
    GenericArg {
        // span of `first` in `fn foo(ns) = first(ns);`
        call: Span,

        // span of `T` in `fn first<T>`
        generic: Span,
    },

    // It's kinda type variable, but the compiler will not try to unify this.
    // It's for "a type equation that is too difficult for the compiler to solve".
    // Let's say `x.y` and `z` has the same type, and it doesn't know the type of `x`.
    // It's too difficult to solve the type of `z`, so the compiler just returns
    // `Type::Blocked { origin: x.def_span }`. It just (temporarily) gives up the inference.
    // When the type-analysis is complete, the compiler checks if it has ever encountered
    // `Type::Blocked`. If so, it does the type-analysis again. Since it has more information
    // than before, it may successfully infer/check all the types without encountering
    // `Type::Blocked` again.
    Blocked {
        origin: Span,
    },
}

#[derive(Clone, Debug)]
pub struct TypeAssertion {
    pub name_span: Span,
    pub type_span: Span,
    pub r#type: Type,
}

impl Type {
    pub fn from_hir(hir_type: &hir::Type, session: &mut Session) -> Result<Type, ()> {
        match hir_type {
            hir::Type::Path(path) => {
                // `inter-hir`'s `check_type_annot_path` should guarantee this
                assert!(path.fields.is_empty());

                match path.id.origin {
                    NameOrigin::GenericParam { .. } => Ok(Type::GenericParam {
                        def_span: path.id.def_span,
                        span: path.id.span,
                    }),
                    NameOrigin::Local { kind } |
                    NameOrigin::Foreign { kind } => match kind {
                        NameKind::Struct |
                        NameKind::Enum => Ok(Type::Static {
                            def_span: path.id.def_span,
                            span: path.id.span,
                        }),
                        _ => {
                            session.errors.push(Error::todo(92226, &format!("lowering hir type: {hir_type:?}"), hir_type.error_span_wide()));
                            Err(())
                        },
                    },
                    _ => unreachable!(),
                }
            },
            hir::Type::Param { constructor, args: hir_args, group_span } => {
                let mut has_error = false;
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
                        constructor_def_span: constructor.id.def_span,
                        constructor_span: constructor.id.span,
                        args,
                        group_span: *group_span,
                    })
                }
            },
            hir::Type::Tuple { types: hir_types, group_span } => {
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
                    Ok(Type::Tuple {
                        args: types,
                        group_span: *group_span,
                    })
                }
            },
            hir::Type::Func { fn_constructor, group_span, params: hir_params, r#return } => {
                let mut has_error = false;
                let fn_span = fn_constructor.id.span;
                let purity = match fn_constructor.id.def_span {
                    f if f == session.get_lang_item_span("type.Fn") => FuncPurity::Both,
                    f if f == session.get_lang_item_span("type.PureFn") => FuncPurity::Pure,
                    f if f == session.get_lang_item_span("type.ImpureFn") => FuncPurity::Impure,
                    _ => {
                        session.errors.push(Error {
                            kind: ErrorKind::InvalidFnType,
                            spans: fn_span.simple_error(),
                            note: None,
                        });
                        return Err(());
                    },
                };

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
                        fn_span,
                        group_span: *group_span,
                        params,
                        r#return: Box::new(r#return.unwrap()),
                        purity,
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
            Type::GenericParam { .. } |
            Type::Never(_) |
            Type::Blocked { .. } => vec![],
            Type::Tuple { args, .. } | Type::Param { args, .. } => {
                let mut result = vec![];

                for arg in args.iter() {
                    result.extend(arg.get_type_vars());
                }

                result
            },
            Type::Func { r#return, params, .. } => {
                let mut result = r#return.get_type_vars();

                for param in params.iter() {
                    result.extend(param.get_type_vars());
                }

                result
            },
            Type::Var { .. } | Type::GenericArg { .. } => vec![self.clone()],
        }
    }

    pub fn substitute(&mut self, type_var: &Type, r#type: &Type) {
        match self {
            Type::Static { .. } |
            Type::GenericParam { .. } |
            Type::Never(_) |
            Type::Blocked { .. } => {},
            Type::Tuple { args,  .. } | Type::Param { args, .. } => {
                for arg in args.iter_mut() {
                    arg.substitute(type_var, r#type);
                }
            },
            Type::Func { r#return, params, .. } => {
                for param in params.iter_mut() {
                    param.substitute(type_var, r#type);
                }

                r#return.substitute(type_var, r#type);
            },
            Type::Var { .. } | Type::GenericArg { .. } => {
                if self == type_var {
                    *self = r#type.clone();
                }
            },
        }
    }

    pub fn substitute_generic_def(&mut self, call: Span, generics: &[Span]) {
        match self {
            Type::GenericParam { def_span, .. } => {
                if generics.contains(def_span) {
                    *self = Type::GenericArg { call, generic: *def_span };
                }
            },
            Type::Static { .. } |
            Type::Never(_) |
            Type::Var { .. } |
            Type::GenericArg { .. } |
            Type::Blocked { .. } => {},
            Type::Tuple { args, .. } | Type::Param { args, .. } => {
                for arg in args.iter_mut() {
                    arg.substitute_generic_def(call, generics);
                }
            },
            Type::Func { r#return, params, .. } => {
                for param in params.iter_mut() {
                    param.substitute_generic_def(call, generics);
                }

                r#return.substitute_generic_def(call, generics);
            },
        }
    }

    pub fn generic_to_type_var(&mut self) {
        match self {
            Type::GenericParam { def_span, .. } => {
                *self = Type::Var { def_span: *def_span, is_return: false };
            },
            Type::Static { .. } |
            Type::Never(_) |
            Type::Var { .. } |
            Type::GenericArg { .. } |
            Type::Blocked { .. } => {},
            // `T<Int>` doesn't make sense...
            Type::Tuple { args, .. } | Type::Param { args, .. } => {
                for arg in args.iter_mut() {
                    arg.generic_to_type_var();
                }
            },
            Type::Func { r#return, params, .. } => {
                for param in params.iter_mut() {
                    param.generic_to_type_var();
                }

                r#return.generic_to_type_var();
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
    struct_shapes: &HashMap<Span, StructShape>,
    lang_items: &HashMap<String, Span>,
) -> Option<Type> {
    match expr {
        Expr::Ident(id) => types.get(&id.def_span).map(|r#type| r#type.clone()),
        Expr::Constant(Constant::Number { n, .. }) => match n.is_integer {
            true => Some(Type::Static {
                def_span: *lang_items.get("type.Int").unwrap(),
                span: Span::None,
            }),
            false => Some(Type::Static {
                def_span: *lang_items.get("type.Number").unwrap(),
                span: Span::None,
            }),
        },
        Expr::Constant(Constant::String { binary, .. }) => match *binary {
            true => Some(Type::Param {
                constructor_def_span: *lang_items.get("type.List").unwrap(),
                constructor_span: Span::None,
                args: vec![Type::Static {
                    def_span: *lang_items.get("type.Byte").unwrap(),
                    span: Span::None,
                }],
                group_span: Span::None,
            }),
            false => Some(Type::Param {
                constructor_def_span: *lang_items.get("type.List").unwrap(),
                constructor_span: Span::None,
                args: vec![Type::Static {
                    def_span: *lang_items.get("type.Char").unwrap(),
                    span: Span::None,
                }],
                group_span: Span::None,
            }),
        },
        Expr::Constant(Constant::Char { .. }) => Some(Type::Static {
            def_span: *lang_items.get("type.Char").unwrap(),
            span: Span::None,
        }),
        Expr::Constant(Constant::Byte { .. }) => Some(Type::Static {
            def_span: *lang_items.get("type.Byte").unwrap(),
            span: Span::None,
        }),
        Expr::If(r#if) => type_of(&r#if.true_value, types, struct_shapes, lang_items),
        Expr::Match(r#match) => type_of(&r#match.arms[0].value, types, struct_shapes, lang_items),
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

                Some(Type::Tuple {
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

impl Session {
    /// It's used for error messages and `dump_type` function.
    /// Make sure to call `init_span_string_map` before calling this function.
    pub fn render_type(&self, r#type: &Type) -> String {
        match r#type {
            Type::Static { def_span, .. } | Type::GenericParam { def_span, .. } => self.span_to_string(*def_span).unwrap_or_else(|| String::from("????")),
            Type::Tuple { args, .. } => format!(
                "({}{})",
                args.iter().map(
                    |arg| self.render_type(arg)
                ).collect::<Vec<_>>().join(", "),
                if args.len() == 1 { "," } else { "" },
            ),
            Type::Param { constructor_def_span, args, .. } => {
                let args = args.iter().map(
                    |arg| self.render_type(arg)
                ).collect::<Vec<_>>().join(", ");

                if constructor_def_span == self.lang_items.get("type.List").unwrap() {
                    format!("[{args}]")
                }

                else {
                    format!("{}<{args}>", self.span_to_string(*constructor_def_span).unwrap_or_else(|| String::from("????")))
                }
            },
            Type::Func { params, r#return, purity, .. } => format!(
                "{}({}) -> {}",
                match purity {
                    FuncPurity::Pure => "PureFn",
                    FuncPurity::Impure => "ImpureFn",
                    FuncPurity::Both => "Fn",
                },
                params.iter().map(
                    |param| self.render_type(param)
                ).collect::<Vec<_>>().join(", "),
                self.render_type(r#return.as_ref()),
            ),
            Type::Var { .. } |
            Type::GenericArg { .. } |
            Type::Blocked { .. } => String::from("_"),
            Type::Never { .. } => String::from("!"),
        }
    }
}
