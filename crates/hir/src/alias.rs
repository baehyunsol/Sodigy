use crate::{GenericDef, Type, Session};
use sodigy_error::{Error, ErrorKind, Warning, WarningKind};
use sodigy_name_analysis::{
    Counter,
    IdentWithOrigin,
    Namespace,
    NameKind,
    NameOrigin,
    UseCount,
};
use sodigy_parse as ast;
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Alias {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<GenericDef>,
    pub r#type: Type,

    // We have to do cycle checks.
    pub foreign_names: HashMap<InternedString, (NameOrigin, Span /* def_span */)>,
}

impl Alias {
    pub fn from_ast(ast_alias: &ast::Alias, session: &mut Session) -> Result<Alias, ()> {
        let mut has_error = false;
        let mut generic_names = HashMap::new();
        let mut generic_index = HashMap::new();

        for (index, GenericDef { name, name_span }) in ast_alias.generics.iter().enumerate() {
            generic_names.insert(*name, (*name_span, NameKind::Generic, UseCount::new()));
            generic_index.insert(*name, index);
        }

        session.name_stack.push(Namespace::ForeignNameCollector {
            is_func: false,
            foreign_names: HashMap::new(),
        });
        session.name_stack.push(Namespace::Generic {
            names: generic_names,
            index: generic_index,
        });

        let r#type = match Type::from_ast(&ast_alias.r#type, session) {
            Ok(t) => Some(t),
            Err(()) => {
                has_error = true;
                None
            },
        };

        let Some(Namespace::Generic { names, .. }) = session.name_stack.pop() else { unreachable!() };

        for (name, (span, kind, count)) in names.iter() {
            // You can't assert inside a type alias, but you can create a type alias inside an assertion.
            if (!session.is_in_debug_context && count.always == Counter::Never) ||
                (session.is_in_debug_context && count.debug_only == Counter::Never) {
                session.warnings.push(Warning {
                    kind: WarningKind::UnusedName {
                        name: *name,
                        kind: *kind,
                    },
                    spans: span.simple_error(),
                    note: None,
                });
            }
        }

        let Some(Namespace::ForeignNameCollector { foreign_names, .. }) = session.name_stack.pop() else { unreachable!() };

        if has_error {
            Err(())
        }

        else {
            let r#type = r#type.unwrap();
            let mut self_references = vec![];
            find_ids_with_def_span(&r#type, ast_alias.name_span, &mut self_references);

            // `type T = Option<T>;` is an error
            if !self_references.is_empty() {
                let mut error_spans = vec![RenderableSpan {
                    span: ast_alias.name_span,
                    auxiliary: false,
                    note: None,
                }];

                for self_reference in self_references.iter() {
                    error_spans.push(RenderableSpan {
                        span: self_reference.span,
                        auxiliary: true,
                        note: None,
                    });
                }

                session.errors.push(Error {
                    kind: ErrorKind::AliasResolveRecursionLimitReached,
                    spans: error_spans,
                    note: None,
                });
                return Err(());
            }

            Ok(Alias {
                keyword_span: ast_alias.keyword_span,
                name: ast_alias.name,
                name_span: ast_alias.name_span,
                generics: ast_alias.generics.clone(),
                r#type,
                foreign_names,
            })
        }
    }
}

fn find_ids_with_def_span(r#type: &Type, def_span: Span, result: &mut Vec<IdentWithOrigin>) {
    match r#type {
        Type::Identifier(id) |
        Type::Path { id, .. } => {
            if id.def_span == def_span {
                result.push(*id);
            }
        },
        Type::Param { r#type, args, .. } => {
            find_ids_with_def_span(r#type, def_span, result);

            for arg in args.iter() {
                find_ids_with_def_span(arg, def_span, result);
            }
        },
        Type::Tuple { types, .. } => {
            for r#type in types.iter() {
                find_ids_with_def_span(r#type, def_span, result);
            }
        },
        Type::Func { args, r#return, .. } => {
            find_ids_with_def_span(r#return, def_span, result);

            for arg in args.iter() {
                find_ids_with_def_span(arg, def_span, result);
            }
        },
        Type::Wildcard(_) => {},
    }
}
