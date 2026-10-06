use sodigy_bytecode::GlobalLabel;

pub struct Assert {
    pub name: String,
    pub label: GlobalLabel,
    pub is_std: bool,
}

