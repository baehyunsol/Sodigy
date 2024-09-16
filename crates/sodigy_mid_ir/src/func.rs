pub struct Func {
    name: IdentWithSpan,

    // all the local name bindings (names that do not have uids) are
    // stored here
    local_values: Vec<LocalValue>,
    uid: Uid,
}

pub struct LocalValue {
    name: IdentWithSpan,
    value: Expr,

    // iff type annotation for this value exists
    ty: Option<Type>,

    parent_func: Uid,
    parent_scope: Option<Uid>,
    name_binding_type: NameBindingType,

    // parent.local_values[self.index] = self
    index: usize,
}
