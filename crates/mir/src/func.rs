use crate::{Expr, RefCount};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

pub struct Func {
    pub name: InternedString,
    pub name_span: Span,
    pub r#type: Option<Expr>,
    pub value: Expr,

    // def_span -> ref_count map for EVERY identifier in `args`, `value` and `type`
    pub value_name_count: HashMap<Span, RefCount>,
}
