use super::Solver;
use crate::Type;
use crate::error::ErrorContext;
use sodigy_mir::Func;
use sodigy_span::Span;
use std::collections::HashMap;

impl Solver {
    pub fn solve_func(
        &mut self,
        func: &Func,
        types: &mut HashMap<Span, Type>,
        generic_instances: &mut HashMap<(Span, Span), Type>,
    ) -> Result<Type, ()> {
        let infered_type = self.solve_expr(&func.value, types, generic_instances)?;
        let mut span_to_name_map = vec![(func.name_span, func.name)];

        for arg in func.args.iter() {
            span_to_name_map.push((arg.name_span, arg.name));
        }

        let span_to_name_map = span_to_name_map.into_iter().collect::<HashMap<_, _>>();
        let (
            annotated_type,
            error_span,
            extra_error_span,
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
                    func.value.error_span(),
                    func.type_annotation_span,
                    ErrorContext::VerifyTypeAnnotation,
                )
            },

            // even though there's no type annotation at all, the mir pass will create the type annotation
            // e.g. `fn add(x, y) = x + y;` has type `Type::Func { args: [Type::Var(x), Type::Var(y)], return: Type::Var(add) }`
            _ => unreachable!(),
        };

        self.equal(
            &annotated_type,
            &infered_type,
            types,
            generic_instances,
            error_span,
            extra_error_span,
            context,
        )?;

        Ok(infered_type)
    }
}
