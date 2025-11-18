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
        types: &mut HashMap<Span, Type>,
        generic_instances: &mut HashMap<(Span, Span), Type>,
    ) -> (Option<Type>, bool /* has_error */) {
        let (infered_type, mut has_error) = self.solve_expr(&r#let.value, types, generic_instances);

        let (
            annotated_type,
            value_span,
            annotation_span,
            context,
        ) = match types.get(&r#let.name_span) {
            None | Some(Type::Var { .. }) => {
                self.add_type_var(Type::Var { def_span: r#let.name_span, is_return: false }, Some(r#let.name));
                (
                    Type::Var {
                        def_span: r#let.name_span,
                        is_return: false,
                    },
                    r#let.value.error_span(),
                    None,
                    ErrorContext::InferTypeAnnotation,
                )
            },
            Some(annotated_type) => (
                annotated_type.clone(),
                r#let.value.error_span(),
                r#let.type_annotation_span,
                ErrorContext::VerifyTypeAnnotation,
            ),
        };

        let infered_type = match infered_type {
            Some(infered_type) => {
                if let Err(()) = self.solve_subtype(
                    &annotated_type,
                    &infered_type,
                    types,
                    generic_instances,
                    false,
                    annotation_span,
                    Some(value_span),
                    context,
                ) {
                    has_error = true;
                }
            },
            None => {
                has_error = true;
            },
        };

        (Some(annotated_type), has_error)
    }
}
