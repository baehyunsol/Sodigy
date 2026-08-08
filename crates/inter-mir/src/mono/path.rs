use super::Monomorphization;
use crate::Session;
use sodigy_mir::{Dotfish, Type};
use sodigy_name_analysis::{IdentWithOrigin, NameOrigin};
use sodigy_span::Span;
use std::collections::HashSet;

impl Session {
    pub fn monomorphize_id(&self, id: &mut IdentWithOrigin, monomorphization: &Monomorphization) {
        id.span = id.span.monomorphize(monomorphization.id);

        match &id.origin {
            NameOrigin::FuncParam { .. } | NameOrigin::Local { .. } => {
                id.def_span = id.def_span.monomorphize(monomorphization.id);
            },
            _ => {},
        }
    }

    pub fn monomorphize_dotfish(
        &mut self,
        dotfish: &mut Option<Dotfish>,
        wildcard_spans: &HashSet<Span>,
        monomorphization: &Monomorphization,
    ) {
        if let Some(dotfish) = dotfish {
            dotfish.group_span = dotfish.group_span.monomorphize(monomorphization.id);

            for r#type in dotfish.types.iter_mut() {
                match r#type {
                    Type::GenericParam { def_span, .. } => {
                        if let Some(monomorphized_type) = monomorphization.generics.get(def_span) {
                            *r#type = monomorphized_type.clone();
                        }
                    },
                    Type::Var { def_span, .. } if wildcard_spans.contains(def_span) => {
                        *def_span = def_span.monomorphize(monomorphization.id);
                    },
                    _ => {},
                }
            }
        }
    }
}
