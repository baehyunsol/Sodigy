use super::Solver;
use crate::Type;
use crate::error::ErrorContext;
use sodigy_mir::Func;
use sodigy_span::Span;
use std::collections::hash_map::{Entry, HashMap};

impl Solver {
    pub fn solve_func(
        &mut self,
        func: &Func,
        types: &mut HashMap<Span, Type>,
        generic_instances: &mut HashMap<(Span, Span), Type>,
    ) -> Result<Type, ()> {
        let infered_type = self.solve_expr(&func.value, types, generic_instances)?;
        let (
            annotated_type,
            error_span,
            extra_error_span,
            context,
        ) = match types.get(&func.name_span) {
            Some(f @ Type::Func { r#return, .. }) => {
                for def_span in f.get_type_vars() {
                    self.add_type_var(def_span.clone(), Some(func.name));
                    self.add_type_var_ref(def_span, Type::Var { def_span: func.name_span, is_return: true });
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
