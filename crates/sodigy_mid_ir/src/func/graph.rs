use super::LocalValueKey;
use std::collections::HashMap;

pub struct LocalValueGraph {
    references: HashMap<LocalValueKey, LocalValueRef>,
    ref_by: HashMap<LocalValueKey, LocalValueRef>,
    ref_by_ret_val: LocalValueRef,

    // type annotations can reference local values (syntactically), but that's an error (semantically)
    // hir pass is supposed to catch all those errors, but I count it again here because
    // 1. a safe guard
    // 2. I might allow dependent types someday
    ref_by_type_annot: LocalValueRef,
}

// this local value is un-conditionally referenced at least `must` times
// and conditionally referenced at least `cond` times.
// if both are 0, it's guaranteed that this value is not referenced
pub struct LocalValueRef {
    must: u32,
    cond: u32,
}
