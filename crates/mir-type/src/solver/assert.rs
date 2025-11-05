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
        let mut has_error = false;

        if let Ok(assertion_type) = self.solve_expr(&assert.value, types, generic_instances) {
            match assertion_type {
                Type::Static(s) if s == self.get_lang_item_span("type.Bool") => {},
                _ => {
                    if self.equal(
                        &Type::Static(self.get_lang_item_span("type.Bool")),
                        &assertion_type,
                        types,
                        generic_instances,
                        false,
                        None,
                        Some(assert.value.error_span()),
                        ErrorContext::AssertConditionBool,
                    ).is_err() {
                        has_error = true;
                    }
                },
            }
        }

        else {
            has_error = true;
        }

        if let Some(note) = &assert.note {
            if let Ok(note_type) = self.solve_expr(note, types, generic_instances) {
                match note_type {
                    Type::Static(s) if s == self.get_lang_item_span("type.String") => {},
                    _ => {
                        if self.equal(
                            &Type::Static(self.get_lang_item_span("type.Bool")),
                            &note_type,
                            types,
                            generic_instances,
                            false,
                            None,
                            Some(assert.value.error_span()),
                            ErrorContext::AssertConditionBool,
                        ).is_err() {
                            has_error = true;
                        }
                    },
                }
            }

            else {
                has_error = true;
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
