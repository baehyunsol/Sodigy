use crate::MacroKind;
use sodigy_error::Error;

impl MacroKind {
    pub fn check(&self, intermediate_dir: &str) -> Result<(), Vec<Error>> {
        match self {
            MacroKind::IncludeString { path: _ } => Ok(()),
            MacroKind::IncludeBytes { path: _ } => Ok(()),
            MacroKind::TypeName { r#type } => r#type.check(),
            MacroKind::TypeNameOfValue { value } => value.check(intermediate_dir),
            MacroKind::NumberOfVariants { r#type } => r#type.check(),
            MacroKind::NumberOfFields { r#type } => r#type.check(),
            MacroKind::NameOfVariants { r#type } => r#type.check(),
            MacroKind::NameOfFields { r#type } => r#type.check(),
            MacroKind::File => Ok(()),
            MacroKind::ModulePath => Ok(()),
            MacroKind::Line => Ok(()),
            MacroKind::Column => Ok(()),
        }
    }
}
