use crate::{Expr, Namespace, NamespaceKind, Session};
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct Func {
    value: Expr,
    foreign_names: HashSet<(InternedString, Span)>,
}

#[derive(Clone, Debug)]
pub struct CallArg {
    pub keyword: Option<(InternedString, Span)>,
    pub arg: Expr,
}

impl Func {
    pub fn from_ast(ast_func: &ast::Func, session: &mut Session) -> Result<Func, ()> {
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

        let value = match Expr::from_ast(&ast_func.value, session) {
            Ok(v) => v,
            Err(_) => todo!(),
        };

        std::mem::swap(&mut func_args, &mut session.curr_func_args);
        session.name_stack.pop();

        // After swapping, `foreign_names` has the foreign names in the current function.
        // We have to update `session.foreign_names` with the values in `foreign_names` that
        // are foreign to the outer function.
        std::mem::swap(&mut foreign_names, &mut session.foreign_names);
        session.update_foreign_names(&foreign_names);

        Ok(Func {
            value,
            foreign_names,
        })
    }
}
