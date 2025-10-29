use crate::Session;
use sodigy_name_analysis::{IdentWithOrigin, NameOrigin};
use sodigy_parse::{self as ast, Field};
use sodigy_span::Span;
use sodigy_string::InternedString;

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
        let root = match session.find_origin_and_count_usage(ast_use.full_path[0].unwrap_name()) {
            Some((origin, def_span)) if def_span != ast_use.name_span => IdentWithOrigin {
                id: ast_use.full_path[0].unwrap_name(),
                span: ast_use.full_path[0].unwrap_span(),
                origin,
                def_span,
            },
            _ => IdentWithOrigin {
                id: ast_use.full_path[0].unwrap_name(),
                span: ast_use.full_path[0].unwrap_span(),
                origin: NameOrigin::External,
                def_span: Span::None,
            },
        };

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
