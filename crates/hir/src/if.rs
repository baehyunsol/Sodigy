use crate::{Expr, Pattern, Session};
use sodigy_name_analysis::{Namespace, NameKind, UseCount};
use sodigy_parse as ast;
use sodigy_span::Span;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct If {
    pub if_span: Span,
    pub cond: Box<Expr>,
    pub pattern: Option<Pattern>,
    pub else_span: Span,
    pub true_value: Box<Expr>,
    pub false_value: Box<Expr>,
}

impl If {
    pub fn from_ast(ast_if: &ast::If, session: &mut Session) -> Result<If, ()> {
        let mut has_error = false;

        let cond = match Expr::from_ast(&ast_if.cond, session) {
            Ok(cond) => Some(cond),
            Err(()) => {
                has_error = true;
                None
            },
        };

        let pattern = match &ast_if.pattern {
            Some(ast_pattern) => {
                let mut extra_guards = vec![];
                let names = ast_pattern.bound_names().iter().map(
                    |(id, span)| (*id, (*span, NameKind::PatternNameBind, UseCount::new()))
                ).collect();
                session.name_stack.push(Namespace::Pattern { names });

                match Pattern::from_ast(ast_pattern, session, &mut extra_guards) {
                    Ok(pattern) => {
                        if !extra_guards.is_empty() {
                            todo!()  // merge this with `cond`
                        }

                        Some(pattern)
                    },
                    Err(()) => {
                        has_error = true;
                        None
                    },
                }
            },
            None => {
                session.name_stack.push(Namespace::Pattern { names: HashMap::new() });
                None
            },
        };

        let true_value = match Expr::from_ast(&ast_if.true_value, session) {
            Ok(true_value) => Some(true_value),
            Err(()) => {
                has_error = true;
                None
            },
        };

        let Some(Namespace::Pattern { names }) = session.name_stack.pop() else { unreachable!() };
        session.warn_unused_names(&names);

        let false_value = match Expr::from_ast(&ast_if.false_value, session) {
            Ok(false_value) => Some(false_value),
            Err(()) => {
                has_error = true;
                None
            },
        };

        if has_error {
            Err(())
        }

        else {
            Ok(If {
                if_span: ast_if.if_span,
                cond: Box::new(cond.unwrap()),
                pattern,
                else_span: ast_if.else_span,
                true_value: Box::new(true_value.unwrap()),
                false_value: Box::new(false_value.unwrap()),
            })
        }
    }
}
