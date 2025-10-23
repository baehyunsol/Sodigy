use super::Solver;
use crate::Type;
use crate::error::ErrorContext;
use sodigy_mir::Assert;
use sodigy_span::Span;
use std::collections::HashMap;
use crate::preludes::*;

impl Solver {
    pub fn solve_assert(
        &mut self,
        assert: &Assert,
        types: &mut HashMap<Span, Type>,
        generic_instances: &mut HashMap<(Span, Span), Type>,
    ) -> Result<(), ()> {
        let assertion_type = self.solve_expr(&assert.value, types, generic_instances)?;
        match assertion_type {
            Type::Static(Span::Prelude(s)) if s == self.preludes[BOOL] => Ok(()),
            _ => self.equal(
                &Type::Static(Span::Prelude(self.preludes[BOOL])),
                &assertion_type,
                types,
                generic_instances,
                assert.value.error_span(),
                Some(assert.keyword_span),
                ErrorContext::AssertConditionBool,
            ),
        }
    }
}
