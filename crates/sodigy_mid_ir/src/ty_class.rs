use crate::ty::Type;
use sodigy_ast::{
    InfixOp,
    PostfixOp,
    PrefixOp,
};
use sodigy_uid::Uid;
use std::collections::HashMap;

// TODO: don't initialize this multiple times -> use one global table
// you can query type classes using this struct
// for example, if you query (`+`, Int, Int), it gives you the implementation of the integer addition
pub struct TypeClassQuery {
    trait_with_2_args: HashMap<TypeClass, HashMap<Type, HashMap<Type, TypeClassDef>>>,
}

// https://smallcultfollowing.com/babysteps//blog/2016/09/24/intersection-impls/
// https://smallcultfollowing.com/babysteps//blog/2016/09/29/distinguishing-reuse-from-override/
// https://smallcultfollowing.com/babysteps/blog/2016/10/24/supporting-blanket-impls-in-specialization/
// https://aturon.github.io/tech/2017/02/06/specialization-and-coherence/
// https://github.com/purescript/documentation/blob/master/language/Type-Classes.md
// I have to make syntax/semantics of type classes before implementing the compiler...

impl TypeClassQuery {
    pub fn query_2_args(
        &self,
        ty_class: TypeClass,
        ty1: &Type,
        ty2: &Type,
    ) -> Option<&TypeClassDef> {
        // TODO: if ty1 or ty2 is generic, placeholder, or a param with placeholders, they have to be handled specially
        // -> that's a trait solver!
        // For ex, if (ty1, ty2) is (List(Int), List(List(Int))),
        // it has to search for...
        // _. (List(Int), List(List(Int)))
        // _. (List(Int), List(List(Any)))
        // _. (List(Int), List(Any))
        // _. (List(Int), Any)
        // _. (List(Any), List(List(Int)))
        // _. (List(Any), List(List(Any)))
        // _. (List(Any), List(Any))
        // _. (List(Any), Any)
        // _. (Any, List(List(Int)))
        // _. (Any, List(List(Any)))
        // _. (Any, List(Any))
        // _. (Any, Any)
        // in which order?
        // it gets even more complicated when type classes are parametrized, ex) `add_1<T: Add(T, Int, T)>(ns: List(T)): T`
        match self.trait_with_2_args.get(&ty_class) {
            Some(table) => match table.get(ty1) {
                Some(table) => table.get(ty2),
                None => None,
            },
            None => None,
        }
    }
}

pub struct TypeClassDef {
    pub(crate) uid: Uid,

    // For example, `ty` of (`+`, Int, Int) is `Func(Int, Int, Int)`
    pub(crate) ty: Type,
}

#[derive(Eq, Hash, PartialEq)]
pub enum TypeClass {
    ToString,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Gt,
    Lt,
    Ne,
    Ge,
    Le,
    Not,
    Neg,
    Range,
    QuestionMark,  // TODO: any better name?
    Custom( /* TODO: how do I represent one? */ ),
}

impl TypeClass {
    // it returns the uid of the generic impl of this type class
    // for example, `Add.generic_uid()` returns the uid of `Add(T, U, V)`
    // it's a tmp placeholder for MIR exprs. it has to be replaced with the
    // exact uid by the type infer engine
    pub fn generic_uid(&self) -> Uid {
        todo!()
    }
}

impl From<PrefixOp> for TypeClass {
    fn from(op: PrefixOp) -> Self {
        match op {
            PrefixOp::Not => TypeClass::Not,
            PrefixOp::Neg => TypeClass::Neg,
        }
    }
}

impl From<PostfixOp> for TypeClass {
    fn from(op: PostfixOp) -> Self {
        match op {
            PostfixOp::Range => TypeClass::Range,
            PostfixOp::QuestionMark => TypeClass::QuestionMark,
        }
    }
}

impl From<InfixOp> for TypeClass {
    fn from(op: InfixOp) -> Self {
        match op {
            InfixOp::Add => TypeClass::Add,
            InfixOp::Sub => TypeClass::Sub,
            InfixOp::Mul => TypeClass::Mul,
            InfixOp::Div => TypeClass::Div,
            InfixOp::Rem => TypeClass::Rem,
            InfixOp::Eq => TypeClass::Eq,
            InfixOp::Gt => TypeClass::Gt,
            InfixOp::Lt => TypeClass::Lt,
            InfixOp::Ne => TypeClass::Ne,
            InfixOp::Ge => TypeClass::Ge,
            InfixOp::Le => TypeClass::Le,
            _ => todo!(),
        }
    }
}
