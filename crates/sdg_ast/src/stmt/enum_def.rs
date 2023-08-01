use super::{Decorator, FuncDef, VariantDef};
use crate::session::InternedString;
use crate::span::Span;

// it's later converted to multiple `FuncDef`s
pub struct EnumDef {
    def_span: Span,
    pub(crate) name_span: Span,
    pub(crate) name: InternedString,
    pub(crate) decorators: Vec<Decorator>,
    variants: Vec<VariantDef>,
}

impl EnumDef {
    pub fn empty(def_span: Span, name_span: Span, name: InternedString) -> Self {
        EnumDef {
            def_span, name_span, name,
            decorators: vec![],
            variants: vec![],
        }
    }

    pub fn new(def_span: Span, name_span: Span, name: InternedString, variants: Vec<VariantDef>) -> Self {
        EnumDef {
            def_span, name_span, name,
            decorators: vec![],
            variants,
        }
    }

    pub fn to_defs(self) -> Vec<FuncDef> {
        todo!()
    }
}
