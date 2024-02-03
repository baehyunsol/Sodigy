use sodigy_ast::IdentWithSpan;
use sodigy_error::{ErrorContext, UniversalError};

pub fn dependency_not_found(dependency: IdentWithSpan) -> UniversalError {
    UniversalError::new(
        ErrorContext::Unknown,
        false,
        true,
        Some(*dependency.span()),
        format!("module not found: `{}`", dependency.id()),
        String::new(),
    )
}

// when `./foo.sdg` and `./foo/lib.sdg` coexist
pub fn conflicting_dependencies(
    dependency: IdentWithSpan,
    path1: String,
    path2: String,
) -> UniversalError {
    UniversalError::new(
        ErrorContext::Unknown,
        false,
        true,
        Some(*dependency.span()),
        format!("conflict in module `{}`", dependency.id()),
        format!("Both `{path1}` and `{path2}` are valid candidates of module `{}`.", dependency.id()),
    )
}
