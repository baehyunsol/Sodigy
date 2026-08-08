use super::Monomorphization;
use crate::Session;
use sodigy_mir::Type;
use sodigy_span::Span;
use std::collections::HashSet;

impl Session {
    pub fn monomorphize_type(
        &self,
        r#type: &Type,
        wildcard_spans: &HashSet<Span>,
        monomorphization: &Monomorphization,
    ) -> Type {
        let mut new_type = r#type.clone();

        for (generic_param, generic_arg) in monomorphization.generics.iter() {
            new_type.substitute_generic_param(generic_param, generic_arg);
        }

        new_type.monomorphize_wildcards(wildcard_spans, monomorphization.id);
        new_type
    }
}
