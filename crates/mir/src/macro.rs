use crate::{Expr, Session, Type};
use sodigy_hir as hir;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub enum MacroKind {
    Hardcode {
        path: InternedString,
        r#type: Type,
    },
    TypeName {
        r#type: Type,
    },
    TypeNameOfValue {
        value: Expr,
    },
    NumberOfVariants {
        r#type: Type,
    },
    NumberOfFields {
        r#type: Type,
    },
    NameOfVariants {
        r#type: Type,
    },
    NameOfFields {
        r#type: Type,
    },
    File,
    ModulePath,
    Line,
    Column,
}

impl MacroKind {
    pub fn from_hir(hir_macro: &hir::MacroKind, session: &mut Session) -> Result<MacroKind, ()> {
        match hir_macro {
            hir::MacroKind::Hardcode { path, r#type } => Ok(MacroKind::Hardcode { path: *path, r#type: Type::from_hir(r#type, session)? }),
            hir::MacroKind::TypeName { r#type } => Ok(MacroKind::TypeName { r#type: Type::from_hir(r#type, session)? }),
            hir::MacroKind::TypeNameOfValue { value } => Ok(MacroKind::TypeNameOfValue { value: Expr::from_hir(value, session)? }),
            hir::MacroKind::NumberOfVariants { r#type } => Ok(MacroKind::NumberOfVariants { r#type: Type::from_hir(r#type, session)? }),
            hir::MacroKind::NumberOfFields { r#type } => Ok(MacroKind::NumberOfFields { r#type: Type::from_hir(r#type, session)? }),
            hir::MacroKind::NameOfVariants { r#type } => Ok(MacroKind::NameOfVariants { r#type: Type::from_hir(r#type, session)? }),
            hir::MacroKind::NameOfFields { r#type } => Ok(MacroKind::NameOfFields { r#type: Type::from_hir(r#type, session)? }),
            hir::MacroKind::File => Ok(MacroKind::File),
            hir::MacroKind::ModulePath => Ok(MacroKind::ModulePath),
            hir::MacroKind::Line => Ok(MacroKind::Line),
            hir::MacroKind::Column => Ok(MacroKind::Column),
        }
    }

    pub fn macro_name(&self) -> &'static str {
        match self {
            MacroKind::Hardcode { .. } => "hardcode",
            MacroKind::TypeName { .. } => "type_name",
            MacroKind::TypeNameOfValue { .. } => "type_name_of_value",
            MacroKind::NumberOfVariants { .. } => "number_of_variants",
            MacroKind::NumberOfFields { .. } => "number_of_fields",
            MacroKind::NameOfVariants { .. } => "name_of_variants",
            MacroKind::NameOfFields { .. } => "name_of_fields",
            MacroKind::File => "file",
            MacroKind::ModulePath => "module_path",
            MacroKind::Line => "line",
            MacroKind::Column => "column",
        }
    }
}
