use crate::{LogEntry, Session, Type, write_log};
use crate::error::{ErrorContext, TypeError, TypeWarning};
use sodigy_mir::Func;
use std::collections::HashMap;

impl Session {
    pub fn solve_func(&mut self, func: &Func) -> (Option<Type>, bool /* has_error */) {
        let mut impure_calls = vec![];
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
        ) = match self.types.get(&func.name_span) {
            Some(f @ Type::Func { r#return, .. }) => {
                let r#return = r#return.clone();

                for type_var in f.get_type_vars() {
                    let type_var_name = match &type_var {
                        Type::Var { def_span, .. } => span_to_name_map.get(def_span).map(|n| *n),
                        _ => None,
                    };
                    self.add_type_var(type_var.clone(), type_var_name);
                    self.add_type_var_ref(type_var, Type::Var { def_span: func.name_span, is_return: true });
                }

                (
                    r#return,
                    func.value.error_span_wide(),
                    func.type_annot_span,
                    if func.type_annot_span.is_some() {
                        ErrorContext::VerifyTypeAnnot
                    } else {
                        ErrorContext::InferedAgain { type_var: Type::Var { def_span: func.name_span, is_return: true } }
                    },
                )
            },

            // even though there's no type annotation at all, the mir pass will create the type annotation
            // e.g. `fn add(x, y) = x + y;` has type `Type::Func { params: [Type::Var(x), Type::Var(y)], return: Type::Var(add) }`
            _ => unreachable!(),
        };

        let (infered_type, mut has_error) = if func.built_in {
            (None, false)
        } else {
            self.solve_expr(&func.value, &mut impure_calls)
        };

        write_log!(self, LogEntry::SolveFunc {
            func: func.clone(),
            annotated_type: annotated_type.as_ref().clone(),
            infered_type: infered_type.clone(),
        });

        if let Some(infered_type) = infered_type {
            if let Err(()) = self.solve_supertype(
                &annotated_type,
                &infered_type,
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
                self.type_errors.push(TypeError::ImpureCallInPureContext {
                    call_spans: impure_calls,
                    keyword_span: func.keyword_span,
                    context: func.origin.into(),
                });
                has_error = true;
            },
            (false, 0) => {
                self.type_warnings.push(TypeWarning::NoImpureCallInImpureContext {
                    impure_keyword_span: func.impure_keyword_span.unwrap(),
                });
            },
            _ => {},
        }

        (Some(*annotated_type), has_error)
    }
}
