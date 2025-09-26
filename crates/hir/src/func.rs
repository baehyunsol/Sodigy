use crate::{Expr, Namespace, NamespaceKind, Session};
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct Func {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub args: Vec<FuncArgDef>,
    pub value: Expr,
    pub foreign_names: HashSet<(InternedString, Span)>,
}

#[derive(Clone, Debug)]
pub struct FuncArgDef {
    pub name: InternedString,
    pub name_span: Span,
    pub r#type: Option<Expr>,
    pub default_value: Option<Expr>,
}

#[derive(Clone, Debug)]
pub struct CallArg {
    pub keyword: Option<(InternedString, Span)>,
    pub arg: Expr,
}

impl Func {
    pub fn from_ast(ast_func: &ast::Func, session: &mut Session) -> Result<Func, ()> {
        let mut has_error = false;

        // `session.foreign_names` was collecting foreign names in the outer function. But now
        // it has to collect foreign names in the inner function.
        let mut foreign_names = HashSet::new();
        std::mem::swap(&mut foreign_names, &mut session.foreign_names);

        let mut func_args = HashMap::new();
        let mut curr_stack = HashMap::new();

        for (index, arg) in ast_func.args.iter().enumerate() {
            func_args.insert(arg.name, (index, arg.name_span));
            curr_stack.insert(arg.name, arg.name_span);
        }

        std::mem::swap(&mut func_args, &mut session.curr_func_args);
        session.name_stack.push(Namespace::new(NamespaceKind::FuncArg, curr_stack));

        let mut args = Vec::with_capacity(ast_func.args.len());

        for arg in ast_func.args.iter() {
            match FuncArgDef::from_ast(arg, session) {
                Ok(arg) => {
                    args.push(arg);
                },
                Err(_) => {
                    has_error = true;
                },
            }
        }

        let value = match Expr::from_ast(&ast_func.value, session) {
            Ok(v) => Some(v),
            Err(_) => {
                has_error = true;
                None
            },
        };

        std::mem::swap(&mut func_args, &mut session.curr_func_args);
        session.name_stack.pop();

        // After swapping, `foreign_names` has the foreign names in the current function.
        // We have to update `session.foreign_names` with the values in `foreign_names` that
        // are foreign to the outer function.
        std::mem::swap(&mut foreign_names, &mut session.foreign_names);
        session.update_foreign_names(&foreign_names);

        if has_error {
            Err(())
        }

        else {
            Ok(Func {
                keyword_span: ast_func.keyword_span,
                name: ast_func.name,
                name_span: ast_func.name_span,
                args,
                value: value.unwrap(),
                foreign_names,
            })
        }
    }
}

impl FuncArgDef {
    pub fn from_ast(ast_arg: &ast::FuncArgDef, session: &mut Session) -> Result<FuncArgDef, ()> {
        let mut r#type = None;
        let mut default_value = None;
        let mut has_error = false;

        if let Some(ast_type) = &ast_arg.r#type {
            match Expr::from_ast(ast_type, session) {
                Ok(t) => {
                    r#type = Some(t);
                },
                Err(_) => {
                    has_error = false;
                },
            }
        }

        if let Some(ast_default_value) = &ast_arg.default_value {
            match Expr::from_ast(ast_default_value, session) {
                Ok(v) => {
                    default_value = Some(v);
                },
                Err(_) => {
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
