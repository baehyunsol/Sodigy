use super::Solver;
use crate::Type;
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
            _ => panic!("TODO: {pattern:?}"),
        }
    }
}
