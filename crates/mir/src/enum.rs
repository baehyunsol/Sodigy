use crate::Session;
use sodigy_hir::{self as hir, Generic};
use sodigy_span::Span;
use sodigy_string::InternedString;

// `session.types` already has all the necessary information, so this
// struct only has names, which are required if you want to dump mir.
#[derive(Clone, Debug)]
pub struct Enum {
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<Generic>,
}

impl Enum {
    pub fn from_hir(hir_enum: &hir::Enum, session: &mut Session) -> Result<Enum, ()> {
        // TODO: How is it gonna lower variants?
        // 1. It has to remember `EnumFieldKind` of each variant.
        // 2. In order to build span_string_map, it has to collect
        //    `(InternedString, Span)` of each variant and of fields
        //    if the variant is of `EnumFieldKind::Struct`
        // 3. There should be a map between variant def_span and enum def_span.
        // 4. If it's `EnumFieldKind::Tuple`, it has to remember
        //    a. number of elements (for `mir::Expr::from_hir`)
        //    b. types of elements (for `inter_mir::type_solver::solve_expr`)
        // 5. If it's `EnumFieldKind::Struct`, it has to remember
        //    a. field names (for `mir::Expr::from_hir`)
        //    b. types of fields (for `inter_mir::type_solver::solve_expr`)
        //       - currently, the compiler does `types.get(&struct.fields[i].name_span)` to get a type of a struct field

        Ok(Enum {
            name: hir_enum.name,
            name_span: hir_enum.name_span.clone(),
            generics: hir_enum.generics.clone(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EnumFieldKind {
    None,
    Tuple,
    Struct,
}
