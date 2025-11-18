use crate::{
    ArgCount,
    ArgType,
    Attribute,
    AttributeKind,
    AttributeRule,
    DecoratorRule,
    Expr,
    Let,
    LetOrigin,
    Poly,
    Requirement,
    Session,
    Type,
    Visibility,
};
use sodigy_error::{Error, ErrorKind};
use sodigy_name_analysis::{
    IdentWithOrigin,
    Namespace,
    NameKind,
    NameOrigin,
    UseCount,
};
use sodigy_parse::{self as ast, GenericDef};
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Func {
    pub visibility: Visibility,
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<GenericDef>,
    pub args: Vec<FuncArgDef>,
    pub r#type: Option<Type>,
    pub value: Expr,
    pub origin: FuncOrigin,
    pub built_in: bool,

    // We have to distinguish closures and lambda functions
    pub foreign_names: HashMap<InternedString, (NameOrigin, Span /* def_span */)>,

    // It only counts `args`.
    // It's later used for optimization.
    pub use_counts: HashMap<InternedString, UseCount>,
}

// TODO: attributes
#[derive(Clone, Debug)]
pub struct FuncArgDef {
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

        let attribute = match session.lower_attribute(
            &ast_func.attribute,
            AttributeKind::Func,
            ast_func.keyword_span,
            is_top_level,
        ) {
            Ok(attribute) => attribute,
            Err(()) => {
                has_error = true;
                Attribute::new()
            },
        };
        let visibility = attribute.visibility.clone();
        let built_in = attribute.get_decorator(b"built_in", &session.intermediate_dir).is_some();
        let any_type = attribute.get_decorator(b"any_type", &session.intermediate_dir).is_some();

        let is_poly = match attribute.get_decorator(b"poly", &session.intermediate_dir) {
            Some(d) => {
                session.polys.insert(ast_func.name_span, Poly {
                    decorator_span: d.name_span,
                    name: ast_func.name,
                    name_span: ast_func.name_span,
                    has_default_impl: ast_func.value.is_some(),
                    impls: vec![],
                });
                true
            },
            None => false,
        };

        let is_impl = match attribute.get_decorator(b"impl", &session.intermediate_dir) {
            Some(d) => {
                session.poly_impls.push((d.args[0].clone(), ast_func.name_span));
                true
            },
            None => false,
        };

        if is_poly || is_impl {
            // TODO: make sure that it has a complete type annotation
            // TODO: warn if it's a poly and has no generic args
        }

        if let Err(()) = session.collect_lang_items(
            &attribute,
            ast_func.name_span,
            Some(&ast_func.generics),
        ) {
            has_error = true;
        }

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

        let value = match &ast_func.value {
            Some(v) => match Expr::from_ast(v, session) {
                // TODO: warn if a built_in func has a body
                Ok(v) => Some(v),
                Err(()) => {
                    has_error = true;
                    None
                },
            },
            None => {
                if is_poly || built_in {
                    // nobody cares!
                    Some(Expr::Char { ch: 0, span: Span::None })
                }

                else {
                    has_error = true;
                    session.errors.push(Error {
                        kind: ErrorKind::FunctionWithoutBody,
                        spans: ast_func.name_span.simple_error(),
                        note: None,
                    });
                    None
                }
            },
        };

        let mut use_counts = HashMap::new();
        let Some(Namespace::FuncArg { names, .. }) = session.name_stack.pop() else { unreachable!() };

        for (name, (_, _, count)) in names.iter() {
            use_counts.insert(*name, *count);
        }

        if ast_func.value.is_some() {
            session.warn_unused_names(&names);
        }

        let Some(Namespace::Generic { names, .. }) = session.name_stack.pop() else { unreachable!() };

        if ast_func.value.is_some() {
            session.warn_unused_names(&names);
        }

        let Some(Namespace::ForeignNameCollector { foreign_names, .. }) = session.name_stack.pop() else { unreachable!() };

        if has_error {
            Err(())
        }

        else {
            Ok(Func {
                visibility,
                keyword_span: ast_func.keyword_span,
                name: ast_func.name,
                name_span: ast_func.name_span,
                generics: ast_func.generics.clone(),
                args,
                r#type,
                value: value.unwrap(),
                origin,
                built_in,
                foreign_names,
                use_counts,
            })
        }
    }

    pub fn get_attribute_rule(is_top_level: bool, is_std: bool, session: &Session) -> AttributeRule {
        let mut attribute_rule = AttributeRule {
            doc_comment: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            doc_comment_error_note: Some(String::from("You can only add doc comments to top-level items.")),
            visibility: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            visibility_error_note: Some(String::from("Only top-level items can be public.")),
            decorators: vec![
                (
                    intern_string(b"poly", &session.intermediate_dir).unwrap(),
                    DecoratorRule {
                        name: intern_string(b"poly", &session.intermediate_dir).unwrap(),
                        requirement: Requirement::Maybe,
                        arg_requirement: Requirement::Never,
                        ..DecoratorRule::default()
                    },
                ), (
                    intern_string(b"impl", &session.intermediate_dir).unwrap(),
                    DecoratorRule {
                        name: intern_string(b"impl", &session.intermediate_dir).unwrap(),
                        requirement: Requirement::Maybe,
                        arg_requirement: Requirement::Must,
                        arg_count: ArgCount::Eq(1),
                        arg_count_error_note: Some(String::from("It can implement exactly 1 poly.")),
                        arg_type: ArgType::Path,
                        arg_type_error_note: Some(String::from("Please specify which poly it implements.")),
                        ..DecoratorRule::default()
                    },
                ),
            ].into_iter().collect(),
        };

        if is_std {
            attribute_rule.add_std_rules(&session.intermediate_dir);
        }

        attribute_rule
    }
}

impl FuncArgDef {
    pub fn from_ast(
        ast_arg: &ast::FuncArgDef,
        session: &mut Session,

        // whether the function or the function-like object is defined in the top-level block
        is_top_level: bool,
    ) -> Result<FuncArgDef, ()> {
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
                        visibility: Visibility::private(),
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
