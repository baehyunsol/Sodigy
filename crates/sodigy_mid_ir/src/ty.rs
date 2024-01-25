use crate::prelude::uids;
use sodigy_uid::Uid;

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Type {
    Solid(Uid),              // Int
    Param(Uid, Vec<Type>),   // Result(Int, Error)
    Generic(/* TODO: how do we represent one? */),

    // `[]: List(Placeholder)`
    // `None: Option(Placeholder)`
    Placeholder,
    HasToBeInfered,
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
