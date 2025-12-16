use sodigy_span::Color;

// TODO: maybe more levels?
pub enum ErrorLevel {
    Error,
    Warning,
}

impl ErrorLevel {
    // NOTE: `ErrorLevel::from_error_kind(k: &ErrorKind)` is implemented in `src/kind.rs` by `error_kinds!()` macro.
    // You can find the actual code in `src/proc_macro.rs`.
    pub fn color(&self) -> Color {
        match self {
            ErrorLevel::Error => Color::Red,
            ErrorLevel::Warning => Color::Yellow,
        }
    }
}
