use super::Solver;
use crate::Type;
use crate::error::ErrorContext;
use sodigy_mir::Let;
use sodigy_span::Span;
use std::collections::HashMap;

impl Solver {
    pub fn solve_let(
        &mut self,
        r#let: &Let,
        impure_calls: &mut Vec<Span>,
        types: &mut HashMap<Span, Type>,
        generic_instances: &mut HashMap<(Span, Span), Type>,
    ) -> (Option<Type>, bool /* has_error */) {
        let (infered_type, mut has_error) = self.solve_expr(
            &r#let.value,
            impure_calls,
            types,
            generic_instances,
        );

        let (
            annotated_type,
            value_span,
            type_annot_span,
            context,
        ) = match types.get(&r#let.name_span) {
            None | Some(Type::Var { .. }) => {
                self.add_type_var(Type::Var { def_span: r#let.name_span, is_return: false }, Some(r#let.name));
                (
                    Type::Var {
                        def_span: r#let.name_span,
                        is_return: false,
                    },
                    r#let.value.error_span_wide(),
                    None,
                    ErrorContext::InferTypeAnnotation,
                )
            },
            Some(annotated_type) => (
                annotated_type.clone(),
                r#let.value.error_span_wide(),
                r#let.type_annot_span,
                ErrorContext::VerifyTypeAnnotation,
            ),
        };

        match infered_type {
            Some(infered_type) => {
                if let Err(()) = self.solve_supertype(
                    &annotated_type,
                    &infered_type,
                    types,
                    generic_instances,
                    false,
                    type_annot_span,
                    Some(value_span),
                    context,

                    // `infered_type` must be subtype of `annotated_type`, but not vice versa.
                    false,
                ) {
                    has_error = true;
                }
            },
            None => {
                has_error = true;
            },
        }

        (Some(annotated_type), has_error)
    }
}
