use crate::{
    Attribute,
    AttributeRule,
    Path,
    Requirement,
    Session,
    Visibility,
    get_decorator_error_notes,
};
use sodigy_error::{Error, ErrorKind, ItemKind};
use sodigy_name_analysis::{IdentWithOrigin, NameOrigin};
use sodigy_parse::{self as ast, Field};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::HashMap;

// If it's `use a.b.c as x;`,
//    name: `x`
//    path: Path { id: a, fields: [b, c] }
// If it's `use a;`
//    name: a
//    path: Path { id: a, fields: [] }
// If it's `use a as x'`
//    name: x
//    path: Path { id: a, fields: [] }
//
// We can track the origin of `a`, but cannot track `b` and `c` (we don't have type info).
// `a` might be defined in the same module, or from an external module.
// If it's external, later name-analysis will find where it's from.
// If `a`'s def_span is itself, it's from an external module.
#[derive(Clone, Debug)]
pub struct Use {
    pub visibility: Visibility,
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub path: Path,
}

impl Use {
    pub fn from_ast(
        ast_use: &ast::Use,
        session: &mut Session,
        is_top_level: bool,
    ) -> Result<Use, ()> {
        let mut has_error = false;
        let (name, span) = match ast_use.full_path[0] {
            Field::Name { name, name_span, .. } => (name, name_span),
            _ => unreachable!(),
        };

        let attribute = match session.lower_attribute(
            &ast_use.attribute,
            ItemKind::Use,
            ast_use.keyword_span,
            is_top_level,
        ) {
            Ok(attribute) => attribute,
            Err(()) => {
                has_error = true;
                Attribute::new()
            },
        };
        let visibility = attribute.visibility.clone();

        let root = match session.find_origin_and_count_usage(name) {
            Some((origin, def_span)) if def_span != ast_use.name_span => IdentWithOrigin {
                id: name,
                span,
                origin,
                def_span,
            },
            _ => {
                let (origin, def_span) = (NameOrigin::External, Span::None);

                IdentWithOrigin {
                    id: name,
                    span,
                    origin,
                    def_span,
                }
            },
        };

        // `use x as x;` is an error
        if root.id == ast_use.name {
            session.errors.push(Error {
                kind: ErrorKind::AliasResolveRecursionLimitReached,
                spans: vec![
                    RenderableSpan {
                        span: ast_use.name_span,
                        auxiliary: false,
                        note: None,
                    },
                    RenderableSpan {
                        span: root.span,
                        auxiliary: true,
                        note: None,
                    },
                ],
                note: None,
            });

            has_error = true;
        }

        if has_error {
            Err(())
        }

        else {
            let fields = if ast_use.full_path.len() > 1 {
                ast_use.full_path[1..].to_vec()
            } else {
                vec![]
            };

            Ok(Use {
                visibility,
                keyword_span: ast_use.keyword_span,
                name: ast_use.name,
                name_span: ast_use.name_span,
                path: Path { id: root, fields },
            })
        }
    }

    pub fn get_attribute_rule(is_top_level: bool, is_std: bool, intermediate_dir: &str) -> AttributeRule {
        let mut attribute_rule = AttributeRule {
            doc_comment: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            doc_comment_error_note: Some(String::from("You can only add doc comments to top-level items.")),
            visibility: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            visibility_error_note: Some(String::from("Only top-level items can be public.")),
            decorators: HashMap::new(),
            decorator_error_notes: get_decorator_error_notes(ItemKind::Use, intermediate_dir),
        };

        if is_std {
            attribute_rule.add_decorators_for_std(ItemKind::Use, intermediate_dir);
        }

        attribute_rule
    }
}
