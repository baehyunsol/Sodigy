use sodigy_ast::IdentWithSpan;
use sodigy_error::{ErrorContext, UniversalError};

pub fn dependency_not_found(dependency: IdentWithSpan) -> UniversalError {
    UniversalError::new(
        ErrorContext::Unknown,
        Some(*dependency.span()),
        false,
        todo!(),
        todo!(),
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
        Some(*dependency.span()),
        false,
        todo!(),
        todo!(),
    )
}
