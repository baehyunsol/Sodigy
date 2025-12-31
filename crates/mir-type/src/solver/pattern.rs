use super::Solver;
use crate::{ErrorContext, Type};
use sodigy_hir::{Pattern, PatternKind};
use sodigy_span::Span;
use std::collections::HashMap;

impl Solver {
    pub fn solve_pattern(
        &mut self,
        pattern: &Pattern,
        types: &mut HashMap<Span, Type>,
        generic_instances: &mut HashMap<(Span, Span), Type>,
    ) -> (Option<Type>, bool /* has_error */) {
        let (pattern_type, has_error) = self.solve_pattern_kind(
            &pattern.kind,
            types,
            generic_instances,
        );

        match (&pattern_type, &pattern.name, &pattern.name_span) {
            // we can solve a type var!
            (Some(pattern_type), Some(name_binding), Some(name_span)) => todo!(),
            _ => {},
        }

        (pattern_type, has_error)
    }

    pub fn solve_pattern_kind(
        &mut self,
        pattern: &PatternKind,
        types: &mut HashMap<Span, Type>,
        generic_instances: &mut HashMap<(Span, Span), Type>,
    ) -> (Option<Type>, bool /* has_error */) {
        match pattern {
            PatternKind::Ident { id, span } => match types.get(span) {
                Some(r#type) => (Some(r#type.clone()), false),
                None => {
                    self.add_type_var(Type::Var { def_span: *span, is_return: false }, Some(*id));
                    (
                        Some(Type::Var {
                            def_span: *span,
                            is_return: false,
                        }),
                        false,
                    )
                },
            },
            PatternKind::Wildcard(span) => match types.get(span) {
                Some(r#type) => (Some(r#type.clone()), false),
                None => {
                    self.add_type_var(Type::Var { def_span: *span, is_return: false }, None);
                    (
                        Some(Type::Var {
                            def_span: *span,
                            is_return: false,
                        }),
                        false,
                    )
                },
            },
            PatternKind::Number { n, .. } => match n.is_integer {
                true => (
                    Some(Type::Static {
                        def_span: self.get_lang_item_span("type.Int"),
                        span: Span::None,
                    }),
                    false,
                ),
                false => (
                    Some(Type::Static {
                        def_span: self.get_lang_item_span("type.Number"),
                        span: Span::None,
                    }),
                    false,
                ),
            },
            PatternKind::String { binary, .. } => match *binary {
                true => (
                    Some(Type::Param {
                        constructor: Box::new(Type::Static {
                            def_span: self.get_lang_item_span("type.List"),
                            span: Span::None,
                        }),
                        args: vec![Type::Static {
                            def_span: self.get_lang_item_span("type.Byte"),
                            span: Span::None,
                        }],
                        group_span: Span::None,
                    }),
                    false,
                ),
                false => (
                    Some(Type::Param {
                        constructor: Box::new(Type::Static {
                            def_span: self.get_lang_item_span("type.List"),
                            span: Span::None,
                        }),
                        args: vec![Type::Static {
                            def_span: self.get_lang_item_span("type.Char"),
                            span: Span::None,
                        }],
                        group_span: Span::None,
                    }),
                    false,
                ),
            },
            PatternKind::Char { .. } => (
                Some(Type::Static {
                    def_span: self.get_lang_item_span("type.Char"),
                    span: Span::None,
                }),
                false,
            ),
            PatternKind::Byte { .. } => (
                Some(Type::Static {
                    def_span: self.get_lang_item_span("type.Byte"),
                    span: Span::None,
                }),
                false,
            ),
            PatternKind::Tuple { elements, rest, .. } => {
                if rest.is_some() {
                    // What can we do?
                    todo!()
                }

                else {
                    let mut elem_types = vec![];
                    let mut has_error = false;

                    for element in elements.iter() {
                        let (elem_type, e) = self.solve_pattern(element, types, generic_instances);
                        has_error |= e;

                        if let Some(elem_type) = elem_type {
                            elem_types.push(elem_type);
                        }
                    }

                    (
                        Some(Type::Param {
                            constructor: Box::new(Type::Unit(Span::None)),
                            args: elem_types,
                            group_span: Span::None,
                        }),
                        has_error,
                    )
                }
            },
            PatternKind::Range { lhs, rhs, .. } => {
                match (
                    lhs.as_ref().map(|lhs| self.solve_pattern(lhs, types, generic_instances)),
                    rhs.as_ref().map(|rhs| self.solve_pattern(rhs, types, generic_instances)),
                ) {
                    (Some(result), None) | (None, Some(result)) => result,
                    (Some((Some(lhs_type), e1)), Some((Some(rhs_type), e2))) => {
                        match self.solve_supertype(
                            &lhs_type,
                            &rhs_type,
                            types,
                            generic_instances,
                            /* is_checking_argument: */ false,
                            Some(lhs.as_ref().unwrap().error_span_wide()),
                            Some(rhs.as_ref().unwrap().error_span_wide()),
                            ErrorContext::RangePatternEqual,
                            /* bidirectional: */ true,
                        ) {
                            Ok(r#type) => (Some(r#type), e1 | e2),
                            Err(()) => (None, true),
                        }
                    },

                    // at least one of these must be an error
                    (Some(_), Some(_)) => (None, true),

                    // parser will reject this
                    (None, None) => unreachable!(),
                }
            },
            PatternKind::Or { lhs, rhs, .. } => {
                // 1. lhs and rhs must have the same type.
                let (pattern_type, mut has_error) = match (
                    self.solve_pattern(lhs, types, generic_instances),
                    self.solve_pattern(rhs, types, generic_instances),
                ) {
                    ((Some(lhs_type), e1), (Some(rhs_type), e2)) => match self.solve_supertype(
                        &lhs_type,
                        &rhs_type,
                        types,
                        generic_instances,
                        /* is_checking_argument: */ false,
                        Some(lhs.error_span_wide()),
                        Some(rhs.error_span_wide()),
                        ErrorContext::OrPatternEqual,
                        /* bidirectional: */ true,
                    ) {
                        Ok(r#type) => (Some(r#type), e1 || e2),
                        Err(()) => (None, true),
                    },
                    _ => (None, true),
                };

                // 2. name bindings in lhs and rhs must have the same type.
                // TODO: If `|` patterns are nested, we don't have to run
                //       this inside inner patterns.
                let mut name_bindings = HashMap::new();

                for (name, name_span) in lhs.bound_names() {
                    name_bindings.insert(name, (name_span, Span::None));
                }

                for (name, name_span) in rhs.bound_names() {
                    name_bindings.get_mut(&name).unwrap().1 = name_span;
                }

                for (name, (lhs_span, rhs_span)) in name_bindings.iter() {
                    let lhs_type_var = Type::Var { def_span: *lhs_span, is_return: false };
                    let rhs_type_var = Type::Var { def_span: *rhs_span, is_return: false };

                    if let Err(()) = self.solve_supertype(
                        &lhs_type_var,
                        &rhs_type_var,
                        types,
                        generic_instances,
                        /* is_checking_argument: */ false,
                        Some(*lhs_span),
                        Some(*rhs_span),
                        ErrorContext::OrPatternNameBinding(*name),
                        /* bidirectional: */ true,
                    ) {
                        has_error = true;
                    }
                }

                (pattern_type, has_error)
            },
            _ => panic!("TODO: {pattern:?}"),
        }
    }
}
