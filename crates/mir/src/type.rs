use crate::{Callable, Expr, GlobalContext, Session};
use sodigy_endec::Endec;
use sodigy_error::{Error, ErrorKind};
use sodigy_hir::{self as hir, FuncPurity};
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_parse::Field;
use sodigy_span::Span;
use sodigy_string::hash;
use sodigy_token::Constant;

// This enum is originally meant for type annotations, but
// type-checker and type-inferer are also using this enum...
//
// `Eq` and `PartialEq` are only for type vars.
// For comparison, use `Solver::equal()` method.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    // `Int`, `Result<Int, String>`, `(Int, Int)`...
    Data {
        // If the type is `Result<Int, String>`, it's `Result`.
        // If the type is `Int`, it's just def_span.
        // If the type is `(Int, Int)`, it's the def_span of `Tuple`
        constructor_def_span: Span,

        // of the type annotation
        constructor_span: Span,

        // If the type is `Result<Int, String>`, it's `[Int, String]`.
        // If the type is `Int`, it's None.
        // If the type is `(Int, Int)`, it's `[Int, Int]`.
        args: Option<Vec<Type>>,

        // If the type is `Result<Int, String>`,
        //    - If there's a type annotation, it's the group span of the angle brackets.
        //    - If there's no type annotation, it's `Some(Span::None)`.
        // If the type is `Int`, it's None.
        // If the type is `(Int, Int)`,
        //     - If there's a type annotation, it's the group span of the parenthesis.
        //     - If there's no type annotation, it's `Some(Span::None)`.
        group_span: Option<Span>,
    },

    Func {
        fn_span: Span,
        group_span: Span,
        params: Vec<Type>,
        r#return: Box<Type>,
        purity: FuncPurity,
    },

    // !
    Never(Span),

    // The first `T` in `fn first<T>(ls: [T]) -> T = ls[0];`
    GenericParam {
        def_span: Span,
        span: Span,
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

                match &path.id.origin {
                    NameOrigin::GenericParam { .. } => Ok(Type::GenericParam {
                        def_span: path.id.def_span.clone(),
                        span: path.id.span.clone(),
                    }),
                    NameOrigin::Local { kind } |
                    NameOrigin::Foreign { kind } => match kind {
                        NameKind::Struct |
                        NameKind::Enum => Ok(Type::Data {
                            constructor_def_span: path.id.def_span.clone(),
                            constructor_span: path.id.span.clone(),
                            args: None,
                            group_span: None,
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
                    Ok(Type::Data {
                        constructor_def_span: constructor.id.def_span.clone(),
                        constructor_span: constructor.id.span.clone(),
                        args: Some(args),
                        group_span: Some(group_span.clone()),
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
                    Ok(Type::Data {
                        constructor_def_span: session.get_lang_item_span("type.Tuple"),
                        constructor_span: group_span.clone(),
                        args: Some(types),
                        group_span: Some(group_span.clone()),
                    })
                }
            },
            hir::Type::Func { fn_constructor, group_span, params: hir_params, r#return } => {
                let mut has_error = false;
                let fn_span = fn_constructor.id.span.clone();
                let purity = match &fn_constructor.id.def_span {
                    f if f == &session.get_lang_item_span("type.Fn") => FuncPurity::Both,
                    f if f == &session.get_lang_item_span("type.PureFn") => FuncPurity::Pure,
                    f if f == &session.get_lang_item_span("type.ImpureFn") => FuncPurity::Impure,
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
                        group_span: group_span.clone(),
                        params,
                        r#return: Box::new(r#return.unwrap()),
                        purity,
                    })
                }
            },

            // it has to be infered
            hir::Type::Wildcard(span) => Ok(Type::Var {
                def_span: span.clone(),
                is_return: false,
            }),
            hir::Type::Never(span) => Ok(Type::Never(span.clone())),
        }
    }

    pub fn get_type_vars(&self) -> Vec<Type> {
        match self {
            Type::GenericParam { .. } |
            Type::Never(_) |
            Type::Blocked { .. } => vec![],
            Type::Data { args, .. } => {
                let mut result = vec![];

                if let Some(args) = args {
                    for arg in args.iter() {
                        result.extend(arg.get_type_vars());
                    }
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

    pub fn has_unsolved_type(&self) -> bool {
        match self {
            Type::Data { args: Some(args), .. } => {
                for arg in args.iter() {
                    if arg.has_unsolved_type() {
                        return true;
                    }
                }

                false
            },
            Type::Func { params, r#return, .. } => {
                for param in params.iter() {
                    if param.has_unsolved_type() {
                        return true;
                    }
                }

                r#return.has_unsolved_type()
            },
            Type::Var { .. } | Type::GenericArg { .. } | Type::Blocked { .. } => true,
            _ => false,
        }
    }

    pub fn substitute(&mut self, type_var: &Type, r#type: &Type) {
        match self {
            Type::GenericParam { .. } |
            Type::Never(_) |
            Type::Blocked { .. } => {},
            Type::Data { args, .. } => {
                if let Some(args) = args {
                    for arg in args.iter_mut() {
                        arg.substitute(type_var, r#type);
                    }
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

    pub fn substitute_generic_param_for_arg(&mut self, call: &Span, generics: &[Span]) {
        match self {
            Type::GenericParam { def_span, .. } => {
                if generics.contains(def_span) {
                    *self = Type::GenericArg { call: call.clone(), generic: def_span.clone() };
                }
            },
            Type::Never(_) |
            Type::Var { .. } |
            Type::GenericArg { .. } |
            Type::Blocked { .. } => {},
            Type::Data { args, .. } => {
                if let Some(args) = args {
                    for arg in args.iter_mut() {
                        arg.substitute_generic_param_for_arg(call, generics);
                    }
                }
            },
            Type::Func { r#return, params, .. } => {
                for param in params.iter_mut() {
                    param.substitute_generic_param_for_arg(call, generics);
                }

                r#return.substitute_generic_param_for_arg(call, generics);
            },
        }
    }

    pub fn substitute_generic_param(&mut self, generic_param: &Span, generic_arg: &Type) {
        match self {
            Type::GenericParam { def_span, .. } if def_span == generic_param => {
                *self = generic_arg.clone();
            },
            Type::GenericParam { .. } |
            Type::Never(_) |
            Type::Var { .. } |
            Type::GenericArg { .. } |
            Type::Blocked { .. } => {},
            Type::Data { args, .. } => {
                if let Some(args) = args {
                    for arg in args.iter_mut() {
                        arg.substitute_generic_param(generic_param, generic_arg);
                    }
                }
            },
            Type::Func { r#return, params, .. } => {
                for param in params.iter_mut() {
                    param.substitute_generic_param(generic_param, generic_arg);
                }

                r#return.substitute_generic_param(generic_param, generic_arg);
            },
        }
    }

    pub fn generic_to_type_var(&mut self) {
        match self {
            Type::GenericParam { def_span, .. } => {
                *self = Type::Var { def_span: def_span.clone(), is_return: false };
            },
            Type::Never(_) |
            Type::Var { .. } |
            Type::GenericArg { .. } |
            Type::Blocked { .. } => {},
            Type::Data { args, .. } => {
                if let Some(args) = args {
                    for arg in args.iter_mut() {
                        arg.generic_to_type_var();
                    }
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

    pub fn type_var_to_generic_param(&mut self) {
        match self {
            Type::Var { def_span, .. } => {
                *self = Type::GenericParam { def_span: def_span.clone(), span: Span::None };
            },
            Type::Never(_) |
            Type::GenericParam { .. } |
            Type::GenericArg { .. } |
            Type::Blocked { .. } => {},
            Type::Data { args, .. } => {
                if let Some(args) = args {
                    for arg in args.iter_mut() {
                        arg.type_var_to_generic_param();
                    }
                }
            },
            Type::Func { r#return, params, .. } => {
                for param in params.iter_mut() {
                    param.type_var_to_generic_param();
                }

                r#return.type_var_to_generic_param();
            },
        }
    }

    /// It's hash of type, not of a type annotation!
    /// It ignores type-annotation-related spans.
    pub fn hash(&self) -> u128 {
        let mut buffer = vec![];

        match self {
            Type::Data { constructor_def_span, args, .. } => {
                buffer.push(0);
                constructor_def_span.encode_impl(&mut buffer);

                if let Some(args) = args {
                    for arg in args.iter() {
                        buffer.extend(arg.hash().to_le_bytes());
                    }
                }
            },
            Type::Func { params, r#return, purity, .. } => {
                buffer.push(1);

                for param in params.iter() {
                    buffer.extend(param.hash().to_le_bytes());
                }

                buffer.extend(r#return.hash().to_le_bytes());
                buffer.push(*purity as u8);
            },
            Type::Never(_) => {
                buffer.push(2);
            },
            Type::GenericParam { def_span, .. } => {
                buffer.push(3);
                def_span.encode_impl(&mut buffer);
            },
            _ => todo!(),
        }

        hash(&buffer)
    }

    pub fn error_span_narrow(&self) -> Span {
        match self {
            Type::Data { constructor_span, .. } => constructor_span.clone(),
            Type::Func { fn_span, .. } => fn_span.clone(),
            Type::Never(span) => span.clone(),
            Type::GenericParam { span, .. } => span.clone(),

            // This function is only for type annotations.
            _ => Span::None,
        }
    }

    pub fn error_span_wide(&self) -> Span {
        match self {
            Type::Data { constructor_span, group_span, .. } => {
                let mut result = constructor_span.clone();

                if let Some(group_span) = &group_span {
                    result = result.merge(group_span);
                }

                result
            },
            Type::Func { fn_span, group_span, r#return, .. } => fn_span
                .merge(group_span)
                .merge(&r#return.error_span_wide()),
            Type::Never(span) => span.clone(),
            Type::GenericParam { span, .. } => span.clone(),

            // This function is only for type annotations.
            _ => Span::None,
        }
    }

    pub fn get_generic_param_def_span(&self) -> Option<Span> {
        match self {
            Type::GenericParam { def_span, .. } => Some(def_span.clone()),
            _ => None,
        }
    }
}

/// It returns the type of `expr`, assuming that type-check and type-infer are
/// complete. If type-check or type-infer is incomplete, it'll return None,
/// or even worse, a wrong type.
pub fn type_of(expr: &Expr, global_context: GlobalContext) -> Option<Type> {
    match expr {
        Expr::Ident { id, dotfish } => {
            assert!(dotfish.is_none());
            global_context.get_type(&id.def_span)
        },
        Expr::Constant(Constant::Number { n, .. }) => match n.is_integer() {
            true => Some(Type::Data {
                constructor_def_span: global_context.get_lang_item_span("type.Int"),
                constructor_span: Span::None,
                args: None,
                group_span: None,
            }),
            false => Some(Type::Data {
                constructor_def_span: global_context.get_lang_item_span("type.Number"),
                constructor_span: Span::None,
                args: None,
                group_span: None,
            }),
        },
        Expr::Constant(Constant::String { binary, .. }) => match *binary {
            true => Some(Type::Data {
                constructor_def_span: global_context.get_lang_item_span("type.List"),
                constructor_span: Span::None,
                args: Some(vec![Type::Data {
                    constructor_def_span: global_context.get_lang_item_span("type.Byte"),
                    constructor_span: Span::None,
                    args: None,
                    group_span: None,
                }]),
                group_span: Some(Span::None),
            }),
            false => Some(Type::Data {
                constructor_def_span: global_context.get_lang_item_span("type.List"),
                constructor_span: Span::None,
                args: Some(vec![Type::Data {
                    constructor_def_span: global_context.get_lang_item_span("type.Char"),
                    constructor_span: Span::None,
                    args: None,
                    group_span: None,
                }]),
                group_span: Some(Span::None),
            }),
        },
        Expr::Constant(Constant::Char { .. }) => Some(Type::Data {
            constructor_def_span: global_context.get_lang_item_span("type.Char"),
            constructor_span: Span::None,
            args: None,
            group_span: None,
        }),
        Expr::Constant(Constant::Byte { .. }) => Some(Type::Data {
            constructor_def_span: global_context.get_lang_item_span("type.Byte"),
            constructor_span: Span::None,
            args: None,
            group_span: None,
        }),
        Expr::Constant(Constant::Scalar(_)) => Some(Type::Data {
            constructor_def_span: global_context.get_lang_item_span("type.Scalar"),
            constructor_span: Span::None,
            args: None,
            group_span: None,
        }),
        Expr::If(r#if) => type_of(&r#if.true_value, global_context),
        Expr::Match(r#match) => type_of(&r#match.arms[0].value, global_context),
        Expr::Block(block) => type_of(&block.value, global_context),
        Expr::Field { lhs, fields, dotfish } => {
            assert!(dotfish.last().unwrap().is_none());
            let Some(lhs_type) = type_of(lhs, global_context.clone()) else { return None; };
            type_of_field(&lhs_type, fields, global_context)
        },
        Expr::FieldUpdate { lhs, .. } => type_of(lhs, global_context),
        Expr::Call { func, args, .. } => match func {
            // TODO: What if it's generic?
            Callable::Static { def_span, .. } => match global_context.get_type(def_span) {
                Some(Type::Func { r#return, .. }) => Some(*r#return.clone()),
                _ => None,
            },

            // If it has type args, there's no way we can figure that out...
            Callable::StructInit { def_span, .. } => todo!(),

            Callable::TupleInit { .. } => {
                let mut arg_types = Vec::with_capacity(args.len());

                for arg in args.iter() {
                    match type_of(arg, global_context.clone()) {
                        Some(t) => { arg_types.push(t); },
                        None => { return None; },
                    }
                }

                Some(Type::Data {
                    constructor_def_span: global_context.get_lang_item_span("type.Tuple"),
                    constructor_span: Span::None,
                    args: Some(arg_types),
                    group_span: Some(Span::None),
                })
            },
            _ => panic!("TODO: {func:?}"),
        },
    }
}

/// Like `type_of`, it has to be used when type-solving is complete and there're no type errors.
pub fn type_of_field(r#type: &Type, field: &[Field], global_context: GlobalContext) -> Option<Type> {
    if field.is_empty() {
        return Some(r#type.clone());
    }

    let t = match r#type {
        Type::Data { constructor_def_span, args, .. } => {
            if *constructor_def_span == global_context.get_lang_item_span("type.Tuple") {
                match &field[0] {
                    Field::Index(i) if *i >= 0 => args.as_ref().unwrap()[*i as usize].clone(),
                    _ => todo!(),
                }
            }

            else if *constructor_def_span == global_context.get_lang_item_span("type.List") {
                match &field[0] {
                    Field::Index(_) => args.as_ref().unwrap()[0].clone(),
                    _ => todo!(),
                }
            }

            else {
                todo!()
            }
        },
        _ => todo!(),
    };

    if field.len() == 1 {
        Some(t)
    }

    else {
        type_of_field(&t, &field[1..], global_context)
    }
}

#[derive(Clone, Debug)]
pub struct Dotfish {
    pub types: Vec<Type>,
    pub group_span: Span,
}

impl Dotfish {
    pub fn from_hir(hir_dotfish: &Option<hir::Dotfish>, session: &mut Session) -> Result<Option<Dotfish>, ()> {
        match hir_dotfish {
            Some(hir::Dotfish { types: hir_types, group_span }) => {
                let mut types = Vec::with_capacity(hir_types.len());
                let mut has_error = false;

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
                    Ok(Some(Dotfish { types, group_span: group_span.clone() }))
                }
            },
            None => Ok(None),
        }
    }
}
