use super::Solver;
use crate::Type;
use crate::error::ErrorContext;
use sodigy_mir::Let;
use sodigy_span::Span;
use std::collections::HashMap;

impl Solver {
    pub fn solve_let(&mut self, r#let: &Let, types: &mut HashMap<Span, Type>) -> Result<Type, ()> {
        let infered_type = self.solve_expr(&r#let.value, types)?;
        let (
            annotated_type,
            error_span,
            extra_error_span,
            context,
        ) = match types.get(&r#let.name_span) {
            None | Some(Type::Var { .. }) => {
                self.add_type_var(r#let.name_span, Some(r#let.name));
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

        self.equal(
            &annotated_type,
            &infered_type,
            types,
            error_span,
            extra_error_span,
            context,
        )?;

        Ok(infered_type)
    }
}
