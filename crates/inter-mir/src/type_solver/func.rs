use crate::{LogId, Session, Type, write_log};
use crate::error::{ErrorContext, TypeError, TypeWarning};
use sodigy_error::{FuncEffect, TypeVarInfo};
use sodigy_mir::Func;
use std::collections::HashMap;

#[cfg(feature = "log")]
use crate::LogEntry;

impl Session {
    pub fn solve_func(&mut self, func: &Func) -> (Option<Type>, bool /* has_error */) {
        let _id = if cfg!(feature = "log") {
            Some(LogId::new())
        } else {
            None
        };

        write_log!(self, LogEntry::SolveFuncStart {
            id: _id.unwrap(),
            func: func.clone(),
        });

        let mut impure_calls = HashMap::new();
        let mut span_to_name_map = vec![(func.name_span.clone(), func.name)];

        for param in func.params.iter() {
            span_to_name_map.push((param.name_span.clone(), param.name));
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
                        Type::Var { def_span, .. } => span_to_name_map.get(def_span).map(|id| TypeVarInfo::Ident(*id)),
                        _ => None,
                    };
                    self.add_type_var(type_var.clone(), type_var_name);
                    self.add_type_var_ref(type_var, Type::Var { def_span: func.name_span.clone(), is_return: true });
                }

                (
                    r#return,
                    func.value.error_span_wide(),
                    func.type_annot_span.clone(),
                    if func.type_annot_span.is_some() {
                        ErrorContext::VerifyTypeAnnot
                    } else {
                        ErrorContext::InferedAgain { type_var: Type::Var { def_span: func.name_span.clone(), is_return: true } }
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

        if let Some(infered_type) = &infered_type {
            if let Err(()) = self.solve_supertype(
                &annotated_type,
                infered_type,
                false,
                type_annot_span.as_ref(),
                Some(&value_span),
                context,

                // `infered_type` must be subtype of `annotated_type`, but not vice versa.
                false,
            ) {
                has_error = true;
            }
        }

        match (&func.effect, impure_calls.len()) {
            (FuncEffect::Fn, 1..) => {
                self.type_errors.push(TypeError::ImpureCallInPureContext {
                    call_spans: impure_calls,
                    keyword_span: func.keyword_span.clone(),
                    context: func.origin.into(),
                    context_effect: func.effect.clone(),
                });
                has_error = true;
            },
            (FuncEffect::Proc, _) => match (impure_calls.get(&FuncEffect::NdetFn), impure_calls.get(&FuncEffect::NdetProc), impure_calls.get(&FuncEffect::Callable)) {
                (None, None, None) => match impure_calls.get(&FuncEffect::Proc) {
                    None if !func.unused_effect => {
                        self.type_warnings.push(TypeWarning::NoImpureCallInImpureContext {
                            effect_keyword_span: func.keyword_span.clone(),
                            context_effect: func.effect.clone(),
                        });
                    },
                    _ => {},
                },
                _ => {
                    let mut impure_calls = impure_calls.clone();
                    impure_calls.remove(&FuncEffect::Proc);
                    self.type_errors.push(TypeError::ImpureCallInPureContext {
                        call_spans: impure_calls,
                        keyword_span: func.keyword_span.clone(),
                        context: func.origin.into(),
                        context_effect: func.effect.clone(),
                    });
                    has_error = true;
                },
            },
            (FuncEffect::NdetFn, _) => match (impure_calls.get(&FuncEffect::Proc), impure_calls.get(&FuncEffect::NdetProc), impure_calls.get(&FuncEffect::Callable)) {
                (None, None, None) => match impure_calls.get(&FuncEffect::NdetFn) {
                    None if !func.unused_effect => {
                        self.type_warnings.push(TypeWarning::NoImpureCallInImpureContext {
                            effect_keyword_span: func.ndet_span.clone().unwrap(),
                            context_effect: func.effect.clone(),
                        });
                    },
                    _ => {},
                },
                _ => {
                    let mut impure_calls = impure_calls.clone();
                    impure_calls.remove(&FuncEffect::NdetFn);
                    self.type_errors.push(TypeError::ImpureCallInPureContext {
                        call_spans: impure_calls,
                        keyword_span: func.keyword_span.clone(),
                        context: func.origin.into(),
                        context_effect: func.effect.clone(),
                    });
                    has_error = true;
                },
            },
            (FuncEffect::NdetProc, 0) if !func.unused_effect => {
                self.type_warnings.push(TypeWarning::NoImpureCallInImpureContext {
                    effect_keyword_span: func.ndet_span.as_ref().unwrap().merge(&func.keyword_span),
                    context_effect: func.effect.clone(),
                });
            },
            _ => {},
        }

        write_log!(self, LogEntry::SolveFuncEnd {
            id: _id.unwrap(),
            annotated_type: annotated_type.as_ref().clone(),
            infered_type: infered_type.clone(),
            has_error,
            last_errors: self.last_errors(),
        });

        (Some(*annotated_type), has_error)
    }
}
