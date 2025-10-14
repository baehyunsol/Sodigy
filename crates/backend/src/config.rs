#[derive(Clone, Debug)]
pub struct CodeGenConfig {
    pub intern_str_map_dir: String,
    pub label_help_comment: bool,
    pub mode: CodeGenMode,
}

#[derive(Clone, Copy, Debug)]
pub enum CodeGenMode {
    Test,
    Bin,
    Lib,
}
