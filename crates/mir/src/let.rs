use crate::{Expr, RefCount};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

pub struct Let {
    pub name: InternedString,
    pub name_span: Span,
    pub r#type: Option<Expr>,
    pub value: Expr,

    // def_span -> ref_count map for EVERY identifier in `value` and `type`
    pub ref_count: HashMap<Span, RefCount>,
}
