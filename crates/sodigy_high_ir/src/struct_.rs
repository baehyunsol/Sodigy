use sodigy_ast as ast;
use sodigy_intern::{InternedString, InternSession};
use sodigy_parse::IdentWithSpan;
use sodigy_uid::Uid;

mod endec;
mod lower;

pub use lower::{lower_ast_struct, name_to_type};

#[derive(Clone)]
pub struct StructInfo {
    pub struct_name: IdentWithSpan,

    // must be sorted according to `sort_struct_fields`
    pub field_names: Vec<InternedString>,
    pub struct_uid: Uid,
    pub constructor_uid: Uid,
}

fn sort_struct_fields(
    fields: &mut Vec<ast::FieldDef>,
    interner: &mut InternSession,
) {
    fields.sort_by_key(|field| interner.unintern_string(field.name.id()).to_vec())
}
