use crate::{
    Attribute,
    AttributeKind,
    AttributeRule,
    Requirement,
    Session,
    Visibility,
};
use sodigy_error::{Error, ErrorKind};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_parse::{self as ast, Field};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::{InternedString, intern_string};
use std::collections::HashMap;

// If it's `use a.b.c as x;`,
//    root: a
//    fields: [b, c]
//    name: `x`
// If it's `use a;`
//    root: a
//    fields: []
//    name: a
// If it's `use a as x'`
//    root: a
//    fields: []
//    name: x
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
    pub fields: Vec<Field>,
    pub root: IdentWithOrigin,
}

impl Use {
    pub fn from_ast(
        ast_use: &ast::Use,
        session: &mut Session,
        is_top_level: bool,
    ) -> Result<Use, ()> {
        let mut has_error = false;
        let (name, span) = match ast_use.full_path[0] {
            Field::Name { name, span, .. } => (name, span),
            _ => unreachable!(),
        };

        let attribute = match session.lower_attribute(
            &ast_use.attribute,
            AttributeKind::Use,
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
                let (origin, def_span) = if name == intern_string(b"std", "").unwrap() {
                    (NameOrigin::Foreign { kind: NameKind::Module }, Span::Std)
                } else if name == intern_string(b"lib", "").unwrap() {
                    (NameOrigin::Foreign { kind: NameKind::Module }, Span::Lib)
                } else {
                    (NameOrigin::External, Span::None)
                };

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
            Ok(Use {
                visibility,
                keyword_span: ast_use.keyword_span,
                name: ast_use.name,
                name_span: ast_use.name_span,
                fields: if ast_use.full_path.len() > 1 {
                    ast_use.full_path[1..].to_vec()
                } else {
                    vec![]
                },
                root,
            })
        }
    }

    pub fn get_attribute_rule(is_top_level: bool, _is_std: bool, _session: &Session) -> AttributeRule {
        AttributeRule {
            doc_comment: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            doc_comment_error_note: Some(String::from("You can only add doc comments to top-level items.")),
            visibility: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            visibility_error_note: Some(String::from("Only top-level items can be public.")),
            decorators: HashMap::new(),
        }
    }
}
