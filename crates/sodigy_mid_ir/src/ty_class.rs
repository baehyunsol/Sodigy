use crate::ty::Type;
use sodigy_ast::InfixOp;
use sodigy_uid::Uid;

// you can query type classes using this struct
// for example, if you query (`+`, Int, Int), it gives you the implementation of the integer addition
pub struct TypeClassQuery {}

impl TypeClassQuery {
    pub fn query_2_args(&self, ty_class: TypeClass, ty1: &Type, ty2: &Type) -> Option<TypeClassDef> {
        todo!()
    }
}

pub struct TypeClassDef {
    pub(crate) uid: Uid,

    // For example, `ty` of (`+`, Int, Int) is `Func(Int, Int, Int)`
    pub(crate) ty: Type,
}

pub enum TypeClass {
    ToString,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Custom( /* TODO: how do I represent one? */ ),
}

impl From<InfixOp> for TypeClass {
    fn from(op: InfixOp) -> Self {
        todo!()
    }
}
