use crate::Session;
use sodigy_error::{Error, ErrorKind};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_parse::{self as ast, Field};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::{InternedString, intern_string};

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
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub fields: Vec<Field>,
    pub root: IdentWithOrigin,
}

impl Use {
    pub fn from_ast(ast_use: &ast::Use, session: &mut Session) -> Result<Use, ()> {
        let (name, span) = match ast_use.full_path[0] {
            Field::Name { name, span, .. } => (name, span),
            _ => unreachable!(),
        };

        let root = match session.find_origin_and_count_usage(ast_use.full_path[0].unwrap_name()) {
            Some((origin, def_span)) if def_span != ast_use.name_span => IdentWithOrigin {
                id: ast_use.full_path[0].unwrap_name(),
                span: ast_use.full_path[0].unwrap_span(),
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
                    id: ast_use.full_path[0].unwrap_name(),
                    span: ast_use.full_path[0].unwrap_span(),
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

            Err(())
        }

        else {
            Ok(Use {
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
}
