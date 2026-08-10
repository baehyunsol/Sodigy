use crate::{Expr, Type};
use sodigy_error::Error;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::Token;

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

pub fn try_parse_macro(
    id: InternedString,
    span: Span,
    group_span: Span,
    group_tokens: &[Token],
) -> Result<Expr, Vec<Error>> {
    match id {
        id if id.eq(b"include_string") => todo!(),
        id if id.eq(b"include_bytes") => todo!(),
        id if id.eq(b"type_name") => todo!(),
        id if id.eq(b"type_name_of_value") => todo!(),
        id if id.eq(b"number_of_variants") => todo!(),
        id if id.eq(b"number_of_fields") => todo!(),
        id if id.eq(b"name_of_variants") => todo!(),
        id if id.eq(b"name_of_fields") => todo!(),
        id if id.eq(b"file") => todo!(),
        id if id.eq(b"module_path") => todo!(),
        id if id.eq(b"line") => todo!(),
        id if id.eq(b"column") => todo!(),
        _ => todo!(),
    }
}
