use sodigy_bytecode::GlobalLabel;

#[derive(Clone, Debug)]
pub struct Assert {
    pub name: String,
    pub label: GlobalLabel,
    pub is_std: bool,
}

#[derive(Clone, Debug)]
pub struct AssertionFilter {
    pub keyword: String,
    pub match_start: bool,  // `^`
    pub match_end: bool,  // `$`
}
