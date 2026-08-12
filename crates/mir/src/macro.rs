use crate::{Expr, Session, Type};
use sodigy_hir as hir;
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
    pub fn from_hir(hir_macro: &hir::MacroKind, session: &mut Session) -> Result<MacroKind, ()> {
        todo!()
    }
}
