use crate::{ErrorContext, LogId, Session, Type, TypeError, write_log};
use sodigy_error::TypeVarInfo;
use sodigy_hir::{Path, Pattern, PatternKind};
use sodigy_mir::{
    byte_type,
    bytes_type,
    char_type,
    get_def_span_from_id,
    int_type,
    number_type,
    string_type,
};
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_span::Span;
use sodigy_token::Constant;
use std::collections::{HashMap, HashSet};

#[cfg(feature = "log")]
use crate::log::LogEntry;

impl Session {
    pub fn solve_pattern(&mut self, pattern: &Pattern) -> (Option<Type>, bool /* has_error */) {
        let _id = if cfg!(feature = "log") {
            Some(LogId::new())
        } else {
            None
        };

        write_log!(self, LogEntry::SolvePatternStart {
            id: _id.unwrap(),
            pattern: pattern.clone(),
        });

        let (result, has_error) = self.solve_pattern_(pattern);

        write_log!(self, LogEntry::SolvePatternEnd {
            id: _id.unwrap(),
            infered_type: result.clone(),
            type_vars: if let Some(r#type) = &result { self.collect_type_var_info(r#type) } else { HashMap::new() },
            has_error,
            last_errors: self.last_errors(),
        });

        (result, has_error)
    }

    // TODO: I don't want to raise errors if the compiler fails to infer type of name bindings in patterns.
    fn solve_pattern_(&mut self, pattern: &Pattern) -> (Option<Type>, bool /* has_error */) {
        let (pattern_type, mut has_error) = self.solve_pattern_kind(&pattern.kind);

        match (&pattern_type, &pattern.name, &pattern.name_span) {
            // we can solve a type var!
            (Some(pattern_type), Some(name_binding), Some(name_span)) => {
                let type_var = Type::Var { def_span: name_span.clone(), is_return: false };
                self.add_type_var(type_var.clone(), Some(TypeVarInfo::Ident(*name_binding)));

                if let Err(()) = self.solve_supertype(
                    pattern_type,
                    &type_var,
                    /* is_checking_argument: */ false,
                    Some(&pattern.error_span_wide()),
                    Some(name_span),
                    ErrorContext::Deep,
                    /* bidirectional: */ true,
                ) {
                    has_error = true;
                }
            },
            _ => {},
        }

        (pattern_type, has_error)
    }

    pub fn solve_pattern_kind(&mut self, pattern: &PatternKind) -> (Option<Type>, bool /* has_error */) {
        match pattern {
            PatternKind::Path(Path { id, .. }) => self.solve_path(id, &None),
            PatternKind::Constant(Constant::Number { n, .. }) => match n.is_integer() {
                true => (Some(int_type(&self.lang_items)), false),
                false => (Some(number_type(&self.lang_items)), false),
            },
            PatternKind::Constant(Constant::String { binary, .. }) => match *binary {
                true => (Some(bytes_type(&self.lang_items)), false),
                false => (Some(string_type(&self.lang_items)), false),
            },
            PatternKind::Constant(Constant::Char { .. }) => (Some(char_type(&self.lang_items)), false),
            PatternKind::Constant(Constant::Byte { .. }) => (Some(byte_type(&self.lang_items)), false),
            PatternKind::NameBinding { id, span, .. } => match self.types.get(span) {
                Some(r#type) => (Some(r#type.clone()), false),
                None => {
                    self.add_type_var(Type::Var { def_span: span.clone(), is_return: false }, Some(TypeVarInfo::Ident(*id)));
                    (
                        Some(Type::Var {
                            def_span: span.clone(),
                            is_return: false,
                        }),
                        false,
                    )
                },
            },
            PatternKind::Struct { r#struct, fields, rest, .. } => {
                let mut field_types = HashMap::with_capacity(fields.len());
                let mut has_error = false;

                for field in fields.iter() {
                    let (r#type, e) = self.solve_pattern(&field.pattern);
                    has_error |= e;

                    if let Some(r#type) = r#type {
                        if let Some((prev_span, _)) = field_types.insert(field.name, (field.span.clone(), r#type)) {
                            // This field appears twice in the pattern, hence an error.
                            todo!();
                        }
                    }
                }

                if let (Some(struct_type), _) = self.solve_path(&r#struct.id, &None) {
                    let (struct_type, struct_shape) = match &struct_type {
                        ty @ Type::Data { constructor_def_span, args, .. } => {
                            let struct_def_span = get_def_span_from_id(*constructor_def_span, args);
                            (ty.clone(), self.struct_shapes.get(&struct_def_span).unwrap())
                        },
                        // An enum variant is solved to this type.
                        Type::Func { r#return, .. } => {
                            let variant_def_span = &r#struct.id.def_span;
                            (*r#return.clone(), self.struct_shapes.get(variant_def_span).unwrap())
                        },
                        _ => unreachable!(),
                    };
                    let mut type_vars_to_add = vec![];
                    let mut missing_fields = vec![];

                    for field in struct_shape.fields.clone().iter() {
                        match field_types.get(&field.name) {
                            Some((pattern_span, infered_type)) => {
                                let mut annotated_type = self.types.get(&field.name_span).unwrap().clone();
                                let mut substituted_generics = HashSet::new();
                                let field_span = field.name_span.clone();
                                annotated_type.substitute_generic_param_for_arg(pattern_span, &mut substituted_generics);

                                for def_span in substituted_generics.iter() {
                                    let type_var = Type::GenericArg { call: pattern_span.clone(), generic: def_span.clone() };

                                    if let Some(already_infered) = self.generic_args.get(&(pattern_span.clone(), def_span.clone())) {
                                        annotated_type.substitute(&type_var, already_infered);
                                    }

                                    type_vars_to_add.push(type_var);
                                }

                                if let Err(()) = self.solve_supertype(
                                    &annotated_type,
                                    infered_type,
                                    false,
                                    Some(&field_span),
                                    Some(pattern_span),
                                    ErrorContext::StructFields,
                                    false,
                                ) {
                                    has_error = true;
                                }
                            },
                            None if rest.is_some() => {},
                            None => {
                                missing_fields.push(field.name);
                            },
                        }
                    }

                    if !missing_fields.is_empty() {
                        has_error = true;
                        self.type_errors.push(TypeError::MissingStructFields {
                            span: r#struct.id.span.clone(),
                            struct_name: r#struct.id.id,
                            is_enum_variant: false,  // TODO: there's no way to check this...
                            missing_fields,
                        });
                    }

                    for type_var in type_vars_to_add.into_iter() {
                        self.add_type_var(type_var, None);
                    }

                    (Some(struct_type), has_error)
                }

                else {
                    // HIR already checked this
                    unreachable!()
                }
            },
            // `Option.Some` has type `Fn(T) -> Option<T>` and the type must already be registered.
            PatternKind::TupleStruct { r#struct, elements, rest, .. } => match self.solve_path(&r#struct.id, &None) {
                (Some(Type::Func { params, r#return, .. }), mut has_error) => {
                    if let NameOrigin::Local { kind: NameKind::EnumVariant } | NameOrigin::Foreign { kind: NameKind::EnumVariant } = &r#struct.id.origin {
                        self.call_to_variant_span.insert(r#struct.id.span.clone(), r#struct.id.def_span.clone());
                    }

                    let mut elem_types = Vec::with_capacity(elements.len());

                    for element in elements.iter() {
                        let (elem_type, e) = self.solve_pattern(element);
                        has_error |= e;

                        match elem_type {
                            Some(elem_type) => {
                                elem_types.push(elem_type);
                            },
                            None => {},
                        }
                    }

                    if has_error {
                        return (None, true);
                    }

                    match (rest, elements.len() == params.len()) {
                        (None, false) => {
                            self.type_errors.push(TypeError::WrongNumberOfArgs {
                                expected: params.to_vec(),
                                got: elem_types.to_vec(),
                                given_keyword_args: vec![],
                                call: r#struct.id.span.clone(),
                                def: Some(r#struct.id.def_span.clone()),
                                arg_spans: elements.iter().map(|element| element.error_span_wide()).collect(),
                            });
                            return (None, true);
                        },
                        (None, true) => {},

                        // TODO: insert type vars to `elem_types`
                        (Some(_), false) => todo!(),

                        // `Some(3, ..)` -> is this a type error?
                        (Some(_), true) => todo!(),  // type-error?
                    }

                    for (i, (param_type, elem_type)) in params.iter().zip(elem_types.iter()).enumerate() {
                        if let Err(()) = self.solve_supertype(
                            elem_type,
                            param_type,
                            false,
                            Some(&elements[i].error_span_wide()),
                            None,
                            ErrorContext::None,
                            false,
                        ) {
                            has_error = true;
                        }
                    }

                    (Some(*r#return.clone()), has_error)
                },
                _ => unreachable!(),
            },
            PatternKind::Tuple { elements, rest, .. } => {
                if rest.is_some() {
                    // What can we do?
                    todo!()
                }

                else {
                    let mut elem_types = vec![];
                    let mut has_error = false;

                    for element in elements.iter() {
                        let (elem_type, e) = self.solve_pattern(element);
                        has_error |= e;

                        if let Some(elem_type) = elem_type {
                            elem_types.push(elem_type);
                        }
                    }

                    (
                        Some(Type::Data {
                            constructor_def_span: self.get_lang_item_span_id("type.Tuple"),
                            constructor_span: Span::None,
                            args: Some(elem_types),
                            group_span: Some(Span::None),
                        }),
                        has_error,
                    )
                }
            },
            PatternKind::List { elements, group_span, rest } => {
                let mut rest_pattern_name_binding = None;
                let type_var = Type::GenericArg { call: group_span.clone(), generic: self.get_lang_item_span("built_in.init_list.generic.0") };
                self.add_type_var(type_var.clone(), Some(TypeVarInfo::ListPattern));

                if let Some(rest) = rest {
                    rest_pattern_name_binding = rest.name_span.clone();
                }

                let (mut r#type, mut has_error) = if elements.is_empty() {
                    let r#type = Type::Data {
                        constructor_def_span: self.get_lang_item_span_id("type.List"),
                        constructor_span: Span::None,
                        args: Some(vec![type_var]),

                        // this is for the type annotation, hence None
                        group_span: Some(Span::None),
                    };
                    (r#type, false)
                }

                else {
                    let mut elem_types = vec![];
                    let mut has_error = false;

                    for element in elements.iter() {
                        let (elem_type, e) = self.solve_pattern(element);
                        has_error |= e;

                        if let Some(elem_type) = elem_type {
                            elem_types.push(elem_type);
                        }
                    }

                    if has_error {
                        return (None, true);
                    }

                    let mut elem_type = elem_types[0].clone();

                    for i in 1..elem_types.len() {
                        if let Ok(new_elem_type) = self.solve_supertype(
                            &elem_type,
                            &elem_types[i],
                            false,
                            Some(&elements[0].error_span_wide()),
                            Some(&elements[i].error_span_wide()),
                            ErrorContext::ListElementEqual,
                            true,
                        ) {
                            elem_type = new_elem_type;
                        }

                        else {
                            has_error = true;
                        }
                    }

                    // It won't return an error. I just want to
                    // register the type-var.
                    if let Err(()) = self.solve_supertype(
                        &elem_type,
                        &type_var,
                        false,
                        None,
                        None,
                        ErrorContext::None,
                        true,
                    ) {
                        has_error = true;
                    }

                    let r#type = Type::Data {
                        constructor_def_span: self.get_lang_item_span_id("type.List"),
                        constructor_span: Span::None,
                        args: Some(vec![elem_type]),

                        // this is for the type annotation, hence None
                        group_span: Some(Span::None),
                    };
                    (r#type, has_error)
                };

                // If there's a rest pattern, it must have the same type.
                if let Some(rest) = rest_pattern_name_binding {
                    let type_var = Type::Var { def_span: rest, is_return: false };
                    self.add_type_var(type_var.clone(), None);

                    if let Ok(new_type) = self.solve_supertype(
                        &type_var,
                        &r#type,
                        false,
                        None,
                        None,
                        ErrorContext::ListElementEqual,
                        true,
                    ) {
                        r#type = new_type;
                    }

                    else {
                        has_error = true;
                    }
                }

                (Some(r#type), has_error)
            },
            PatternKind::Range { lhs, rhs, .. } => {
                match (
                    lhs.as_ref().map(|lhs| self.solve_pattern(lhs)),
                    rhs.as_ref().map(|rhs| self.solve_pattern(rhs)),
                ) {
                    (Some(result), None) | (None, Some(result)) => result,
                    (Some((Some(lhs_type), e1)), Some((Some(rhs_type), e2))) => {
                        match self.solve_supertype(
                            &lhs_type,
                            &rhs_type,
                            /* is_checking_argument: */ false,
                            Some(&lhs.as_ref().unwrap().error_span_wide()),
                            Some(&rhs.as_ref().unwrap().error_span_wide()),
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
                    self.solve_pattern(lhs),
                    self.solve_pattern(rhs),
                ) {
                    ((Some(lhs_type), e1), (Some(rhs_type), e2)) => match self.solve_supertype(
                        &lhs_type,
                        &rhs_type,
                        /* is_checking_argument: */ false,
                        Some(&lhs.error_span_wide()),
                        Some(&rhs.error_span_wide()),
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
                    let lhs_type_var = Type::Var { def_span: lhs_span.clone(), is_return: false };
                    let rhs_type_var = Type::Var { def_span: rhs_span.clone(), is_return: false };
                    self.add_type_var(lhs_type_var.clone(), None);
                    self.add_type_var(rhs_type_var.clone(), None);

                    if let Err(()) = self.solve_supertype(
                        &lhs_type_var,
                        &rhs_type_var,
                        /* is_checking_argument: */ false,
                        Some(lhs_span),
                        Some(rhs_span),
                        ErrorContext::OrPatternNameBinding(*name),
                        /* bidirectional: */ true,
                    ) {
                        has_error = true;
                    }
                }

                (pattern_type, has_error)
            },
            PatternKind::Wildcard(span) => match self.types.get(span) {
                Some(r#type) => (Some(r#type.clone()), false),
                None => {
                    self.add_type_var(Type::Var { def_span: span.clone(), is_return: false }, None);
                    (
                        Some(Type::Var {
                            def_span: span.clone(),
                            is_return: false,
                        }),
                        false,
                    )
                },
            },
            _ => panic!("TODO: {pattern:?}"),
        }
    }
}
