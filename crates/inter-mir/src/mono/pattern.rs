use super::Monomorphization;
use crate::Session;
use sodigy_mir::{Pattern, PatternKind};

impl Session {
    pub fn monomorphize_pattern(&mut self, pattern: &mut Pattern, monomorphization: &Monomorphization) {
        // There's no type annotation in patterns, so we don't have to update type variables.
        if let Some(name_span) = &mut pattern.name_span {
            *name_span = name_span.monomorphize(monomorphization.id);
        }

        match &mut pattern.kind {
            PatternKind::NameBinding { span, .. } => {
                *span = span.monomorphize(monomorphization.id);
            },
            PatternKind::Tuple { elements, rest, group_span } |
            PatternKind::List { elements, rest, group_span } => {
                *group_span = group_span.monomorphize(monomorphization.id);

                for element in elements.iter_mut() {
                    self.monomorphize_pattern(element, monomorphization);
                }

                if let Some(rest) = rest {
                    rest.span = rest.span.monomorphize(monomorphization.id);

                    if let Some(name_span) = &mut rest.name_span {
                        *name_span = name_span.monomorphize(monomorphization.id);
                    }
                }
            },
            PatternKind::Or { lhs, rhs, op_span } => {
                self.monomorphize_pattern(lhs, monomorphization);
                self.monomorphize_pattern(rhs, monomorphization);
                *op_span = op_span.monomorphize(monomorphization.id);
            },
            PatternKind::Wildcard(span) => {
                *span = span.monomorphize(monomorphization.id);
            },
            _ => panic!("TODO: {pattern:?}"),
        }
    }
}
