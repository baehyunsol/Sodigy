#[derive(Clone, Debug)]
pub struct CodeGenConfig {
    // You need this when you call `unintern_string`.
    pub intermediate_dir: String,

    // If it's set, it adds comments to the generated code.
    pub label_help_comment: bool,

    pub mode: CodeGenMode,
}

#[derive(Clone, Copy, Debug)]
pub enum CodeGenMode {
    // Runs the tests (top-level assertions) and prints the result.
    // If any test fails, the program must terminate with non-zero exit code.
    Test,

    // Calls the main function.
    Binary,

    // You can generate mir of a library, but cannot code-gen a library.
    // Library,
}
