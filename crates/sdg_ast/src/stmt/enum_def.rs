use super::{Decorator, FuncDef, GenericDef, VariantDef};
use crate::session::InternedString;
use crate::span::Span;

// it's later converted to multiple `FuncDef`s
pub struct EnumDef {
    def_span: Span,
    generics: Vec<GenericDef>,
    pub(crate) name_span: Span,
    pub(crate) name: InternedString,
    pub(crate) decorators: Vec<Decorator>,
    variants: Vec<VariantDef>,
}

impl EnumDef {
    pub fn empty(def_span: Span, name_span: Span, name: InternedString, generics: Vec<GenericDef>) -> Self {
        EnumDef {
            def_span, name_span, name,
            decorators: vec![],
            variants: vec![],
            generics,
        }
    }

    pub fn new(def_span: Span, name_span: Span, name: InternedString, variants: Vec<VariantDef>, generics: Vec<GenericDef>) -> Self {
        EnumDef {
            def_span, name_span, name,
            decorators: vec![],
            variants,
            generics,
        }
    }

    /*
     * Enum Foo { A, B(Int, Int, String) }
     *
     * # kind: Enum
     * def Foo: Type = new_enum().variant_num(2);
     *
     * # add `Foo` to its path
     * # kind: EnumVariant
     * def A: Foo = Foo.variant(0);
     *
     * # add `Foo` to its path
     * # kind: EnumVariant
     * def B(e1: Int, e2: Int, e3: String): Foo = Foo.variant(1, (e1, e2, e3));
     */
    pub fn to_defs(self) -> Vec<FuncDef> {
        todo!()
    }
}
