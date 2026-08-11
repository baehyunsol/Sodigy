use crate::{Expr, Session, Type};
use sodigy_parse as ast;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub enum MacroKind {
    IncludeString {
        path: InternedString,
    },
    IncludeBytes {
        path: InternedString,
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
    pub fn from_ast(ast_macro: &ast::MacroKind, session: &mut Session) -> Result<MacroKind, ()> {
        match ast_macro {
            ast::MacroKind::IncludeString { path } => Ok(MacroKind::IncludeString { path: *path }),
            ast::MacroKind::IncludeBytes { path } => Ok(MacroKind::IncludeBytes { path: *path }),
            ast::MacroKind::TypeName { r#type } => Ok(MacroKind::TypeName { r#type: Type::from_ast(r#type, session)? }),
            ast::MacroKind::TypeNameOfValue { value } => Ok(MacroKind::TypeNameOfValue { value: Expr::from_ast(value, session)? }),
            ast::MacroKind::NumberOfVariants { r#type } => Ok(MacroKind::NumberOfVariants { r#type: Type::from_ast(r#type, session)? }),
            ast::MacroKind::NumberOfFields { r#type } => Ok(MacroKind::NumberOfFields { r#type: Type::from_ast(r#type, session)? }),
            ast::MacroKind::NameOfVariants { r#type } => Ok(MacroKind::NameOfVariants { r#type: Type::from_ast(r#type, session)? }),
            ast::MacroKind::NameOfFields { r#type } => Ok(MacroKind::NameOfFields { r#type: Type::from_ast(r#type, session)? }),
            ast::MacroKind::File => Ok(MacroKind::File),
            ast::MacroKind::ModulePath => Ok(MacroKind::ModulePath),
            ast::MacroKind::Line => Ok(MacroKind::Line),
            ast::MacroKind::Column => Ok(MacroKind::Column),
        }
    }

    pub fn macro_name(&self) -> &'static str {
        match self {
            MacroKind::IncludeString { .. } => "include_string",
            MacroKind::IncludeBytes { .. } => "include_bytes",
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
