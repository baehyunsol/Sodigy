use super::Monomorphization;
use crate::Session;
use sodigy_mir::Type;

impl Session {
    pub fn monomorphize_type(&mut self, r#type: &Type, monomorphization: &Monomorphization) -> Type {
        let mut new_type = r#type.clone();

        for (generic_param, generic_arg) in monomorphization.generics.iter() {
            new_type.substitute_generic_param(generic_param, generic_arg);
        }

        new_type
    }
}
