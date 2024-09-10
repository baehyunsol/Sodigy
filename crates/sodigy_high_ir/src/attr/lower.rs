use super::{Attribute, Decorator};
use crate::{HirSession, IdentWithOrigin, NameSpace, concat_doc_comments, lower_ast_expr};
use sodigy_ast as ast;
use sodigy_intern::InternedString;
use sodigy_parse::IdentWithSpan;
use sodigy_span::SpanRange;
use std::collections::{HashMap, HashSet};

pub fn lower_ast_decorator(
    decorator: &ast::Decorator,
    session: &mut HirSession,
    used_names: &mut HashSet<IdentWithOrigin>,
    imports: &HashMap<InternedString, (SpanRange, Vec<IdentWithSpan>)>,
    name_space: &mut NameSpace,
) -> Result<Decorator, ()> {
    let mut has_error = false;

    let args = if let Some(args) = &decorator.args {
        let mut result = Vec::with_capacity(args.len());

        for arg in args.iter() {
            if let Ok(arg) = lower_ast_expr(
                arg,
                session,
                used_names,
                imports,
                name_space,
            ) {
                result.push(arg);
            }

            else {
                has_error = true;
            }
        }

        Some(result)
    } else {
        None
    };

    if has_error {
        Err(())
    }

    else {
        Ok(Decorator {
            name: decorator.name.clone(),
            args,
        })
    }
}

pub fn lower_ast_attributes(
    attributes: &Vec<ast::Attribute>,
    session: &mut HirSession,
    used_names: &mut HashSet<IdentWithOrigin>,
    imports: &HashMap<InternedString, (SpanRange, Vec<IdentWithSpan>)>,
    name_space: &mut NameSpace,
) -> Result<Vec<Attribute>, ()> {
    let mut doc_comments = vec![];
    let mut result = Vec::with_capacity(attributes.len());
    let mut has_error = false;

    for attribute in attributes.iter() {
        match attribute {
            ast::Attribute::DocComment(d) => {
                doc_comments.push(*d);
            }
            ast::Attribute::Decorator(d) => {
                if let Ok(d) = lower_ast_decorator(
                    d,
                    session,
                    used_names,
                    imports,
                    name_space,
                ) {
                    result.push(Attribute::Decorator(d));
                }

                else {
                    has_error = true;
                }
            },
        }
    }

    if let Some(d) = concat_doc_comments(&doc_comments, session) {
        result.push(Attribute::DocComment(d));
    }

    if has_error {
        Err(())
    }

    else {
        Ok(result)
    }
}
