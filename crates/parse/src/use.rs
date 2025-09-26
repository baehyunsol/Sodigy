use sodigy_span::Span;
use sodigy_string::InternedString;

// TODO: syntax
// `use a.b.c;`
// `use a.b.c as d;`
// `use a.b.{c as d, e as f, g.h.i};`

#[derive(Clone, Debug)]
pub struct Use {
    pub keyword_span: Span,
    // use <full_path> as <name>
    pub name: InternedString,
    pub name_span: Span,
    pub full_path: Vec<(InternedString, Span)>,
}
