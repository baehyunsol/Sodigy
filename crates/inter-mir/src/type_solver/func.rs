use super::TypeSolver;
use crate::Type;
use crate::error::{ErrorContext, TypeError, TypeWarning};
use sodigy_mir::Func;
use sodigy_span::Span;
use std::collections::HashMap;

impl TypeSolver {
    pub fn solve_func(
        &mut self,
        func: &Func,
        types: &mut HashMap<Span, Type>,
        generic_instances: &mut HashMap<(Span, Span), Type>,
    ) -> (Option<Type>, bool /* has_error */) {
        let mut impure_calls = vec![];
        let (infered_type, mut has_error) = self.solve_expr(
            &func.value,
            &mut impure_calls,
            types,
            generic_instances,
        );
        let mut span_to_name_map = vec![(func.name_span, func.name)];

        for param in func.params.iter() {
            span_to_name_map.push((param.name_span, param.name));
        }

        let span_to_name_map = span_to_name_map.into_iter().collect::<HashMap<_, _>>();
        let (
            annotated_type,
            value_span,
            type_annot_span,
            context,
        ) = match types.get(&func.name_span) {
            Some(f @ Type::Func { r#return, .. }) => {
                for type_var in f.get_type_vars() {
                    let Type::Var { def_span, .. } = &type_var else { unreachable!() };
                    self.add_type_var(type_var.clone(), span_to_name_map.get(def_span).map(|n| *n));
                    self.add_type_var_ref(type_var, Type::Var { def_span: func.name_span, is_return: true });
                }

                (
                    r#return.clone(),
                    func.value.error_span_wide(),
                    func.type_annot_span,
                    ErrorContext::VerifyTypeAnnotation,
                )
            },

            // even though there's no type annotation at all, the mir pass will create the type annotation
            // e.g. `fn add(x, y) = x + y;` has type `Type::Func { params: [Type::Var(x), Type::Var(y)], return: Type::Var(add) }`
            _ => unreachable!(),
        };

        if let Some(infered_type) = infered_type {
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
        }

        match (func.is_pure, impure_calls.len()) {
            (true, 1..) => {
                self.errors.push(TypeError::ImpureCallInPureContext {
                    call_spans: impure_calls,
                    keyword_span: func.keyword_span,
                    context: func.origin.into(),
                });
                has_error = true;
            },
            (false, 0) => {
                self.warnings.push(TypeWarning::NoImpureCallInImpureContext {
                    impure_keyword_span: func.impure_keyword_span.unwrap(),
                });
            },
            _ => {},
        }

        (Some(*annotated_type), has_error)
    }
}
