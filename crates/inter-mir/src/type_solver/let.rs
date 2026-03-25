use crate::{Session, Type, write_log};
use crate::error::ErrorContext;
use sodigy_mir::Let;
use sodigy_span::Span;

#[cfg(feature = "log")]
use crate::LogEntry;

impl Session {
    pub fn solve_let(&mut self, r#let: &Let, impure_calls: &mut Vec<Span>) -> (Option<Type>, bool /* has_error */) {
        let (
            annotated_type,
            value_span,
            type_annot_span,
            context,
        ) = match self.types.get(&r#let.name_span) {
            None | Some(Type::Var { .. }) => {
                let type_var = Type::Var { def_span: r#let.name_span.clone(), is_return: false };
                self.add_type_var(type_var.clone(), Some(r#let.name));
                (
                    type_var,
                    r#let.value.error_span_wide(),
                    None,
                    ErrorContext::InferTypeAnnot,
                )
            },
            Some(annotated_type) => (
                annotated_type.clone(),
                r#let.value.error_span_wide(),
                r#let.type_annot_span.clone(),
                if r#let.type_annot_span.is_some() {
                    ErrorContext::VerifyTypeAnnot
                } else {
                    ErrorContext::InferedAgain { type_var: Type::Var { def_span: r#let.name_span.clone(), is_return: false } }
                },
            ),
        };

        let (infered_type, mut has_error) = self.solve_expr(&r#let.value, impure_calls);
        write_log!(self, LogEntry::SolveLet {
            r#let: r#let.clone(),
            annotated_type: annotated_type.clone(),
            infered_type: infered_type.clone(),
        });

        match infered_type {
            Some(infered_type) => {
                if let Err(()) = self.solve_supertype(
                    &annotated_type,
                    &infered_type,
                    false,
                    type_annot_span.as_ref(),
                    Some(&value_span),
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
