use crate::{Expr, FuncArgDef, Session, Type};
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;

pub struct Struct {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub fields: Vec<StructField<Type>>,
}

pub type StructField<T> = FuncArgDef<T>;

#[derive(Clone, Debug)]
pub struct StructInitField {
    pub name: InternedString,
    pub name_span: Span,
    pub value: Expr,
}

impl Struct {
    pub fn from_ast(ast_struct: &ast::Struct, session: &mut Session) -> Result<Struct, ()> {
        let mut fields = Vec::with_capacity(ast_struct.fields.len());
        let mut has_error = false;

        for field in ast_struct.fields.iter() {
            match StructField::from_ast(field, session) {
                Ok(field) => {
                    fields.push(field);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(Struct {
                keyword_span: ast_struct.keyword_span,
                name: ast_struct.name,
                name_span: ast_struct.name_span,
                fields,
            })
        }
    }
}
