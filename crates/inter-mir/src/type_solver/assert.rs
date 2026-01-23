use super::TypeSolver;
use crate::Type;
use crate::error::ErrorContext;
use sodigy_mir::Assert;
use sodigy_span::Span;
use std::collections::HashMap;

impl TypeSolver {
    pub fn solve_assert(
        &mut self,
        assert: &Assert,
        impure_calls: &mut Vec<Span>,
        types: &mut HashMap<Span, Type>,
        generic_args: &mut HashMap<(Span, Span), Type>,
    ) -> Result<(), ()> {
        let (assertion_type, mut has_error) = self.solve_expr(
            &assert.value,
            impure_calls,
            types,
            generic_args,
        );

        if let Some(assertion_type) = assertion_type {
            if let Err(()) = self.solve_supertype(
                &Type::Static {
                    def_span: self.get_lang_item_span("type.Bool"),
                    span: Span::None,
                },
                &assertion_type,
                types,
                generic_args,
                false,
                None,
                Some(assert.value.error_span_wide()),
                ErrorContext::AssertConditionBool,
                false,
            ) {
                has_error = true;
            }
        }

        if let Some(note) = &assert.note {
            let (note_type, e) = self.solve_expr(
                note,
                impure_calls,
                types,
                generic_args,
            );
            has_error |= e;

            if let Some(note_type) = note_type {
                if let Err(()) = self.solve_supertype(
                    // We shouldn't use `Type::Static { def_span: lang_item("type.String") }` here!!
                    // `String` is just an alias to `[Char]` and it's already resolved.
                    &Type::Param {
                        constructor: Box::new(Type::Static {
                            def_span: self.get_lang_item_span("type.List"),
                            span: Span::None,
                        }),
                        args: vec![Type::Static {
                            def_span: self.get_lang_item_span("type.Char"),
                            span: Span::None,
                        }],
                        group_span: Span::None,
                    },
                    &note_type,
                    types,
                    generic_args,
                    false,
                    None,
                    Some(note.error_span_wide()),
                    ErrorContext::AssertConditionBool,
                    false,
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
