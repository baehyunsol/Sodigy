use super::Monomorphization;
use crate::Session;
use sodigy_mir::{Dotfish, Type};
use sodigy_name_analysis::{IdentWithOrigin, NameOrigin};

impl Session {
    pub fn monomorphize_id(&mut self, id: &mut IdentWithOrigin, monomorphization: &Monomorphization) {
        id.span = id.span.monomorphize(monomorphization.id);

        match &id.origin {
            NameOrigin::FuncParam { .. } | NameOrigin::Local { .. } => {
                id.def_span = id.def_span.monomorphize(monomorphization.id);
            },
            _ => {},
        }
    }

    pub fn monomorphize_dotfish(&mut self, dotfish: &mut Option<Dotfish>, monomorphization: &Monomorphization) {
        if let Some(dotfish) = dotfish {
            dotfish.group_span = dotfish.group_span.monomorphize(monomorphization.id);

            for r#type in dotfish.types.iter_mut() {
                if let Type::GenericParam { def_span, .. } = r#type {
                    if let Some(monomorphized_type) = monomorphization.generics.get(def_span) {
                        *r#type = monomorphized_type.clone();
                    }
                }
            }
        }
    }
}
