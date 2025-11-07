use crate::{
    Expr,
    Let,
    LetOrigin,
    Public,
    Session,
    StdAttribute,
    Type,
};
use sodigy_error::{Warning, WarningKind};
use sodigy_name_analysis::{
    Counter,
    IdentWithOrigin,
    Namespace,
    NameKind,
    NameOrigin,
    UseCount,
};
use sodigy_parse::{self as ast, GenericDef};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Func {
    pub public: Public,
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<GenericDef>,
    pub args: Vec<FuncArgDef<Type>>,
    pub r#type: Option<Type>,
    pub value: Expr,
    pub origin: FuncOrigin,
    pub std_attribute: StdAttribute,

    // We have to distinguish closures and lambda functions
    pub foreign_names: HashMap<InternedString, (NameOrigin, Span /* def_span */)>,

    // It only counts `args`.
    // It's later used for optimization.
    pub use_counts: HashMap<InternedString, UseCount>,
}

#[derive(Clone, Debug)]
pub struct FuncArgDef<Type> {
    pub name: InternedString,
    pub name_span: Span,
    pub r#type: Option<Type>,

    // `fn foo(x = 3, y = bar()) = ...;` is lowered to
    // `let foo_default_x = 3; let foo_default_y = bar(); fn foo(x = foo_default_x, y = foo_default_y) = ...;`
    pub default_value: Option<IdentWithOrigin>,
}

#[derive(Clone, Copy, Debug)]
pub enum FuncOrigin {
    TopLevel,
    Inline,  // `fn` keyword in an inline block
    Lambda,
}

#[derive(Clone, Debug)]
pub struct CallArg {
    pub keyword: Option<(InternedString, Span)>,
    pub arg: Expr,
}

impl Func {
    pub fn from_ast(
        ast_func: &ast::Func,
        session: &mut Session,
        origin: FuncOrigin,
        is_top_level: bool,
    ) -> Result<Func, ()> {
        let mut has_error = false;

        let std_attribute = match StdAttribute::from_ast(&ast_func.attribute, session) {
            Ok(a) => a,
            Err(()) => {
                has_error = true;
                StdAttribute::new()
            },
        };

        let public = match Public::from_ast(&ast_func.attribute.public, session) {
            Ok(p) => Some(p),
            Err(()) => {
                has_error = true;
                None
            },
        };

        let mut func_arg_names = HashMap::new();
        let mut func_arg_index = HashMap::new();
        let mut generic_names = HashMap::new();
        let mut generic_index = HashMap::new();

        for (index, arg) in ast_func.args.iter().enumerate() {
            func_arg_names.insert(arg.name, (arg.name_span, NameKind::FuncArg, UseCount::new()));
            func_arg_index.insert(arg.name, index);
        }

        for (index, generic) in ast_func.generics.iter().enumerate() {
            generic_names.insert(generic.name, (generic.name_span, NameKind::Generic, UseCount::new()));
            generic_index.insert(generic.name, index);
        }

        session.name_stack.push(Namespace::ForeignNameCollector {
            is_func: true,
            foreign_names: HashMap::new(),
        });
        session.name_stack.push(Namespace::Generic {
            names: generic_names,
            index: generic_index,
        });

        // We have to lower args before pushing args to the name_stack because
        // 1. Sodigy doesn't allow dependent types.
        // 2. An arg's default value should not reference other args.
        let mut args = Vec::with_capacity(ast_func.args.len());

        for arg in ast_func.args.iter() {
            match FuncArgDef::from_ast(arg, session, is_top_level) {
                Ok(arg) => {
                    args.push(arg);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        session.name_stack.push(Namespace::FuncArg {
            names: func_arg_names,
            index: func_arg_index,
        });

        let mut r#type = None;

        if let Some(ast_type) = &ast_func.r#type {
            match Type::from_ast(&ast_type, session) {
                Ok(ty) => {
                    r#type = Some(ty);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        let value = match Expr::from_ast(&ast_func.value, session) {
            Ok(v) => Some(v),
            Err(()) => {
                has_error = true;
                None
            },
        };

        let mut use_counts = HashMap::new();
        let Some(Namespace::FuncArg { names, .. }) = session.name_stack.pop() else { unreachable!() };

        for (name, (span, kind, count)) in names.iter() {
            use_counts.insert(*name, *count);

            if (!session.is_in_debug_context && count.always == Counter::Never) ||
                (session.is_in_debug_context && count.debug_only == Counter::Never) {
                let mut note = None;

                if count.debug_only != Counter::Never {
                    note = Some(String::from("This value is only used in debug mode."));
                }

                session.warnings.push(Warning {
                    kind: WarningKind::UnusedName {
                        name: *name,
                        kind: *kind,
                    },
                    spans: span.simple_error(),
                    note,
                });
            }
        }

        let Some(Namespace::Generic { names, .. }) = session.name_stack.pop() else { unreachable!() };

        for (name, (span, kind, count)) in names.iter() {
            if (!session.is_in_debug_context && count.always == Counter::Never) ||
                (session.is_in_debug_context && count.debug_only == Counter::Never) {
                let mut note = None;

                if count.debug_only != Counter::Never {
                    note = Some(String::from("This value is only used in debug mode."));
                }

                session.warnings.push(Warning {
                    kind: WarningKind::UnusedName {
                        name: *name,
                        kind: *kind,
                    },
                    spans: span.simple_error(),
                    note,
                });
            }
        }

        let Some(Namespace::ForeignNameCollector { foreign_names, .. }) = session.name_stack.pop() else { unreachable!() };

        if has_error {
            Err(())
        }

        else {
            Ok(Func {
                public: public.unwrap(),
                keyword_span: ast_func.keyword_span,
                name: ast_func.name,
                name_span: ast_func.name_span,
                generics: ast_func.generics.clone(),
                args,
                r#type,
                value: value.unwrap(),
                origin,
                std_attribute,
                foreign_names,
                use_counts,
            })
        }
    }
}

impl FuncArgDef<Type> {
    pub fn from_ast(
        ast_arg: &ast::FuncArgDef,
        session: &mut Session,

        // whether the function or the function-like object is defined in the top-level block
        is_top_level: bool,
    ) -> Result<FuncArgDef<Type>, ()> {
        let mut r#type = None;
        let mut default_value = None;
        let mut has_error = false;

        if let Some(ast_type) = &ast_arg.r#type {
            match Type::from_ast(ast_type, session) {
                Ok(t) => {
                    r#type = Some(t);
                },
                Err(()) => {
                    has_error = false;
                },
            }
        }

        if let Some(ast_default_value) = &ast_arg.default_value {
            session.name_stack.push(Namespace::ForeignNameCollector {
                is_func: false,
                foreign_names: HashMap::new(),
            });

            match Expr::from_ast(ast_default_value, session) {
                Ok(v) => {
                    let Some(Namespace::ForeignNameCollector { foreign_names, .. }) = session.name_stack.pop() else { unreachable!() };
                    session.push_func_default_value(Let {
                        public: Public::private(),
                        keyword_span: Span::None,
                        name: ast_arg.name,
                        name_span: ast_arg.name_span,
                        r#type: r#type.clone(),
                        value: v,
                        origin: LetOrigin::FuncDefaultValue,
                        foreign_names,
                    });

                    default_value = Some(IdentWithOrigin {
                        id: ast_arg.name,
                        span: ast_arg.name_span,
                        origin: NameOrigin::Local {
                            kind: NameKind::Let { is_top_level },
                        },
                        def_span: ast_arg.name_span,
                    });
                },
                Err(()) => {
                    session.name_stack.pop();
                    has_error = false;
                },
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(FuncArgDef {
                name: ast_arg.name,
                name_span: ast_arg.name_span,
                r#type,
                default_value,
            })
        }
    }
}
