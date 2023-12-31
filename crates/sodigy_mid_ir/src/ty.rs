use crate::prelude::uids;
use sodigy_uid::Uid;

#[derive(Clone)]
pub enum Type {
    Solid(Uid),
    Param(Uid, Vec<Type>),
    Generic(/* TODO: how do we represent one? */),
}

impl Type {
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
