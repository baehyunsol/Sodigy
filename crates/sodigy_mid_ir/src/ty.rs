use crate::expr::{Expr, ExprKind};
use crate::prelude::uids;
use sodigy_uid::Uid;

mod endec;

#[derive(Clone, Hash)]
pub enum Type {
    Solid(Uid),              // Int
    Param(Uid, Vec<Type>),   // Result(Int, Error)

    // for now, using an integer index to identify `Generic` makes sense because
    // every `Type` belongs to exactly one `Def`.
    Generic(usize),

    // `[]: List(Placeholder)`
    // `None: Option(Placeholder)`
    Placeholder,
    HasToBeInfered,

    // in Sodigy, Types are first class objects.
    // that means the language doesn't distinguish types and exprs.
    // but the compiler does so for the sake of efficiency
    //
    // this variant has to be lowered to Type::Solid or Type::Param
    // before the type-checking pass
    HasToBeConverted(Box<Expr>),
}

impl Type {
    pub fn is_known(&self) -> bool {
        match self {
            Type::Solid(_) => true,
            Type::Param(_, args) => args.iter().all(|ty| ty.is_known()),
            _ => false,
        }
    }

    pub fn is_list_of(&self) -> Option<&Self> {
        match self {
            // List(T) -> T
            Type::Param(
                ty,
                gen,
            ) if *ty == uids::LIST_DEF && gen.len() == 1 => gen.get(0),
            _ => None,
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Solid(uid1), Type::Solid(uid2)) => uid1 == uid2,
            (Type::Param(uid1, args1), Type::Param(uid2, args2)) => uid1 == uid2 && args1 == args2,

            // TODO: how do I compare 2 generics?
            // for example, 2 `T`s in `let get<T>(v: List(T), i: Int): T` are the same.
            // but if the `get` function is instantiated multiple times with different `T`s,
            // `T`s from different instances are different, but how do we distinguish them?
            // they both belong to the same function and has the same index..
            (Type::Generic(..), Type::Generic(..)) => todo!(),

            // there's no point in comparing `Type::HasToBeInfered`
            _ => false,
        }
    }
}

impl Eq for Type {}

// https://doc.rust-lang.org/nomicon/subtyping.html
// https://baehyunsol.github.io/CoqStudy/Chap15-4.html
//
// let's say `Student` is subtype of `Person`
//
// `let s: Person = Student { .. }` is okay
// `let f: Func(Student, Person) = \{p: Person, Student { .. }}` is okay
pub fn is_subtype_of(
    sup: &Type,
    sub: &Type,
) -> bool {
    match (sup, sub) {
        // `let s: List(Int) = []` is okay
        // `Placeholder` is subtype of every type
        (_, Type::Placeholder) => true,

        // There's no implicit type casting in Sodigy
        (Type::Solid(sup), Type::Solid(sub)) => sup == sub,
        (Type::Param(sup, sup_param), Type::Param(sub, sub_param)) if sub == sup && sup_param.len() == sub_param.len() => {
            if *sub == uids::FUNC_DEF {
                todo!()
            }

            else {
                (0..sup_param.len()).all(
                    |i| is_subtype_of(&sup_param[i], &sub_param[i])
                )
            }
        },

        // TODO: generics
        _ => false,
    }
}

// All the type annotations are expressions in Hir
// those expressions are first lowered to Mir::Expr, then to Mir::Type.
// This function tries the conversion, and returns None if it fails.
// It only collects the low-hanging fruits, like `Int` and `String`.
pub fn try_convert_expr_to_ty(expr: &Expr) -> Option<Type> {
    match &expr.kind {
        ExprKind::Global(uid) => todo!(),
        _ => todo!(),
    }
}
