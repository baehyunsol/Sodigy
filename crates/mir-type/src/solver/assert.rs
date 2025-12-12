use super::Solver;
use crate::Type;
use crate::error::ErrorContext;
use sodigy_mir::Assert;
use sodigy_span::Span;
use std::collections::HashMap;

impl Solver {
    pub fn solve_assert(
        &mut self,
        assert: &Assert,
        types: &mut HashMap<Span, Type>,
        generic_instances: &mut HashMap<(Span, Span), Type>,
    ) -> Result<(), ()> {
        let (assertion_type, mut has_error) = self.solve_expr(&assert.value, types, generic_instances);

        if let Some(assertion_type) = assertion_type {
            if let Err(()) = self.solve_subtype(
                &Type::Static {
                    def_span: self.get_lang_item_span("type.Bool"),
                    span: Span::None,
                },
                &assertion_type,
                types,
                generic_instances,
                false,
                None,
                Some(assert.value.error_span()),
                ErrorContext::AssertConditionBool,
            ) {
                has_error = true;
            }
        }

        if let Some(note) = &assert.note {
            let (note_type, e) = self.solve_expr(note, types, generic_instances);
            has_error |= e;

            if let Some(note_type) = note_type {
                if let Err(()) = self.solve_subtype(
                    &Type::Static {
                        def_span: self.get_lang_item_span("type.String"),
                        span: Span::None,
                    },
                    &note_type,
                    types,
                    generic_instances,
                    false,
                    None,
                    Some(assert.value.error_span()),
                    ErrorContext::AssertConditionBool,
                ) {
                    has_error = true;
                }
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }
}
