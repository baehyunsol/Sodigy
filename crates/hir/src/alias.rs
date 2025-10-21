use crate::{Type, Session};
use sodigy_error::{Warning, WarningKind};
use sodigy_name_analysis::{Counter, Namespace, NameKind, NameOrigin, UseCount};
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

pub struct Alias {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub args: Vec<(InternedString, Span)>,
    pub r#type: Type,

    // We have to do cycle checks.
    pub foreign_names: HashMap<InternedString, (NameOrigin, Span /* def_span */)>,
}

impl Alias {
    pub fn from_ast(ast_alias: &ast::Alias, session: &mut Session) -> Result<Alias, ()> {
        let mut has_error = false;
        let mut arg_names = HashMap::new();
        let mut arg_index = HashMap::new();

        for (index, (name, name_span)) in ast_alias.args.iter().enumerate() {
            arg_names.insert(*name, (*name_span, NameKind::Generic, UseCount::new()));
            arg_index.insert(*name, index);
        }

        session.name_stack.push(Namespace::ForeignNameCollector {
            is_func: false,
            foreign_names: HashMap::new(),
        });
        session.name_stack.push(Namespace::Generic {
            names: arg_names,
            index: arg_index,
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
                    span: *span,
                    ..Warning::default()
                });
            }
        }

        let Some(Namespace::ForeignNameCollector { foreign_names, .. }) = session.name_stack.pop() else { unreachable!() };

        if has_error {
            Err(())
        }

        else {
            Ok(Alias {
                keyword_span: ast_alias.keyword_span,
                name: ast_alias.name,
                name_span: ast_alias.name_span,
                args: ast_alias.args.clone(),
                r#type: r#type.unwrap(),
                foreign_names,
            })
        }
    }
}
