use crate::Session;
use crate::error::{ErrorContext, TypeError};
use sodigy_mir::{Dotfish, Type};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_span::Span;
use std::collections::HashSet;

impl Session {
    pub fn solve_path(&mut self, id: &IdentWithOrigin, dotfish: &Option<Dotfish>) -> (Option<Type>, bool /* has_error */) {
        match self.types.get(&id.def_span) {
            Some(r#type) => {
                let mut r#type = r#type.clone();
                let mut substituted_generics = HashSet::new();
                r#type.substitute_generic_param_for_arg(&id.span, &mut substituted_generics);

                for def_span in substituted_generics.iter() {
                    self.add_type_var(Type::GenericArg { call: id.span.clone(), generic: def_span.clone() }, None);
                }

                (Some(r#type), false)
            },
            None => {
                match &id.origin {
                    NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => match kind {
                        NameKind::EnumVariant { .. } | NameKind::Struct => {
                            let def_span = match kind {
                                // `False` in `Bool.False` has type `Bool`.
                                NameKind::EnumVariant { parent } => parent,
                                NameKind::Struct => &id.def_span,
                                _ => unreachable!(),
                            };
                            let item_shape = match self.get_item_shape(def_span) {
                                Some(item_shape) => {
                                    if item_shape.generics().is_empty() {
                                        let has_error = if let Some(dotfish) = dotfish {
                                            self.type_errors.push(TypeError::WrongNumberOfGenericArgs {
                                                expected: 0,
                                                got: dotfish.types.len(),
                                                param_group_span: Span::None,
                                                arg_group_span: dotfish.group_span.clone(),
                                            });
                                            true
                                        } else {
                                            false
                                        };

                                        return (
                                            Some(Type::Data {
                                                constructor_def_span: def_span.clone(),
                                                constructor_span: Span::None,
                                                args: None,
                                                group_span: None,
                                            }),
                                            has_error,
                                        );
                                    }

                                    else {
                                        let mut dotfish_group_span = Span::None;
                                        let mut has_error = false;
                                        let type_args: Vec<Type> = item_shape.generics().iter().map(
                                            |generic| Type::GenericArg {
                                                call: id.span.clone(),
                                                generic: generic.name_span.clone(),
                                            }
                                        ).collect();

                                        if let Some(dotfish) = dotfish {
                                            dotfish_group_span = dotfish.group_span.clone();

                                            if dotfish.types.len() != item_shape.generics().len() {
                                                self.type_errors.push(TypeError::WrongNumberOfGenericArgs {
                                                    expected: item_shape.generics().len(),
                                                    got: dotfish.types.len(),
                                                    param_group_span: item_shape.generic_group_span().clone().unwrap_or(Span::None),
                                                    arg_group_span: dotfish.group_span.clone(),
                                                });
                                                return (None, true);
                                            }

                                            for (type_arg_var, type_arg) in type_args.iter().zip(dotfish.types.iter()) {
                                                if let Err(()) = self.solve_supertype(
                                                    type_arg_var,
                                                    type_arg,
                                                    false,
                                                    None,
                                                    Some(&type_arg.error_span_wide()),
                                                    ErrorContext::None,
                                                    true,
                                                ) {
                                                    has_error = true;
                                                }
                                            }
                                        }

                                        for type_arg in type_args.iter() {
                                            self.add_type_var(type_arg.clone(), None);
                                        }

                                        return (
                                            Some(Type::Data {
                                                constructor_def_span: def_span.clone(),
                                                constructor_span: Span::None,
                                                args: Some(type_args),
                                                group_span: Some(dotfish_group_span),
                                            }),
                                            has_error,
                                        );
                                    }
                                },
                                None => todo!(),  // unreachable?
                            };
                        },
                        NameKind::Func => {
                            // If it has generic parameters, do something
                            let func_shape = match self.func_shapes.get(&id.def_span) {
                                _ => todo!(),
                            };

                            todo!()
                        },
                        NameKind::PatternNameBind => {
                            self.pattern_name_bindings.insert(id.def_span.clone());
                        },
                        _ => {},
                    },
                    _ => {},
                }

                // NOTE: inter-hir must have checked that `id` is a valid expression

                let type_var = Type::Var { def_span: id.def_span.clone(), is_return: false };
                self.add_type_var(type_var.clone(), Some(id.id));
                (Some(type_var), false)
            },
        }
    }
}
