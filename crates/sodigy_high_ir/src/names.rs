pub enum NameOrigin {
    FuncArg {
        index: usize,
    },
    FuncGeneric {
        index: usize,
    },
    Local,   // match arm, `if let`, scope
    Global,  // `def`, `struct`, `enum`, `module`, `use`, ...
}
