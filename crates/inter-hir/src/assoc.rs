use crate::Session;
use sodigy_error::{Error, ErrorKind};
use sodigy_hir::{
    AssociatedItem,
    AssociatedItemKind,
    Expr,
    Func,
    FuncOrigin,
    FuncParam,
    Generic,
    Path,
    Poly,
    Type,
    Visibility,
};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_span::{PolySpanKind, Span};
use sodigy_string::intern_string;
use std::collections::hash_map::{Entry, HashMap};

impl Session {
    pub fn resolve_associated_items(&mut self) -> Result<(), ()> {
        fn get_def_span(associated_item: &AssociatedItem, r#type: &Type) -> Result<(bool, Span), Error> {
            match r#type {
                Type::Path(path) | Type::Param { constructor: path, .. } => {
                    match path.id.origin {
                        NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => match kind {
                            NameKind::Struct => Ok((true, path.id.def_span)),
                            NameKind::Enum => Ok((false, path.id.def_span)),
                            NameKind::GenericParam => Err(Error {
                                kind: ErrorKind::TooGeneralToAssociateItem,
                                spans: associated_item.type_span.simple_error(),
                                note: None,
                            }),

                            // already filtered out by `check_type_annotation_path`
                            _ => unreachable!(),
                        },

                        // already filtered out by `check_type_annotation_path`
                        _ => unreachable!(),
                    }
                },
                Type::Tuple { .. } => todo!(),  // what's def_span of tuple? maybe use lang_item?
                Type::Func { .. } | Type::Never(_) => Err(Error {
                    kind: ErrorKind::CannotAssociateItem,
                    spans: associated_item.type_span.simple_error(),
                    note: None,
                }),
                Type::Wildcard(_) => Err(Error {
                    kind: ErrorKind::TooGeneralToAssociateItem,
                    spans: associated_item.type_span.simple_error(),
                    note: None,
                }),
            }
        }

        let mut has_error = false;
        let mut associated_items = self.associated_items.drain(..).collect::<Vec<_>>();

        'associated_items: for associated_item in associated_items.iter_mut() {
            if let Err(()) = self.resolve_type(&mut associated_item.r#type, &mut vec![]) {
                has_error = true;
                continue;
            }

            else if let Err(()) = self.check_type_annotation_path(&associated_item.r#type) {
                has_error = true;
                continue;
            }

            match get_def_span(&associated_item, &associated_item.r#type) {
                Ok((is_struct, def_span)) => {
                    if is_struct {
                        let struct_shape = self.struct_shapes.get_mut(&def_span).unwrap();

                        for (associated_item_kind, params, is_pure, name, name_span) in struct_shape.fields.iter().map(
                            |field| (AssociatedItemKind::Field, None, None, field.name, field.name_span)
                        ).chain(struct_shape.associated_funcs.iter().map(
                            // for error messages, `spans[0]` is enough
                            |(name, (params, is_pure, spans))| (AssociatedItemKind::Func, Some(*params), Some(*is_pure), *name, spans[0])
                        )).chain(struct_shape.associated_lets.iter().map(
                            |(name, name_span)| (AssociatedItemKind::Let, None, None, *name, *name_span)
                        )) {
                            if name == associated_item.name {
                                let error = match (associated_item_kind, associated_item.kind) {
                                    (AssociatedItemKind::Field | AssociatedItemKind::Let, AssociatedItemKind::Func) => todo!(),  // err
                                    (_, AssociatedItemKind::Let) => todo!(),  // err
                                    (AssociatedItemKind::Func, AssociatedItemKind::Func) => {
                                        if associated_item.params == params && associated_item.is_pure == is_pure {
                                            // okay
                                            continue;
                                        }

                                        else {
                                            todo!()  // err
                                        }
                                    },
                                    (_, AssociatedItemKind::Field) => unreachable!(),
                                    (AssociatedItemKind::Variant, _) | (_, AssociatedItemKind::Variant) => unreachable!(),
                                };

                                self.errors.push(error);
                                has_error = true;
                                continue 'associated_items;
                            }
                        }

                        if let AssociatedItemKind::Func = associated_item.kind {
                            let params = associated_item.params.unwrap();
                            let is_pure = associated_item.is_pure.unwrap();

                            match struct_shape.associated_funcs.entry(associated_item.name) {
                                Entry::Occupied(mut e) => {
                                    e.get_mut().2.push(associated_item.name_span);
                                },
                                Entry::Vacant(e) => {
                                    e.insert((params, is_pure, vec![associated_item.name_span]));
                                },
                            }

                            let poly_name = format!(
                                "associated_func::{}::{}::{params}",
                                associated_item.name.unintern_or_default(&self.intermediate_dir),
                                if is_pure { "pure" } else { "impure" },
                            );
                            let poly_name_interned = intern_string(poly_name.as_bytes(), &self.intermediate_dir).unwrap();
                            let poly_span: Span = Span::Poly {
                                name: poly_name_interned,
                                kind: PolySpanKind::Name,
                            };

                            match self.new_polys.entry(poly_span) {
                                Entry::Occupied(mut e) => {
                                    e.get_mut().impls.push(associated_item.name_span);
                                },
                                Entry::Vacant(e) => {
                                    let generic_params = (0..(params + 1)).map(
                                        |i| intern_string(
                                            if i != params {
                                                format!("T{i}")
                                            } else {
                                                String::from("V")
                                            }.as_bytes(),
                                            &self.intermediate_dir,
                                        ).unwrap()
                                    ).collect::<Vec<_>>();
                                    let param_names = (0..params).map(
                                        |i| intern_string(format!("p{i}").as_bytes(), &self.intermediate_dir).unwrap()
                                    ).collect::<Vec<_>>();

                                    e.insert(Poly {
                                        decorator_span: Span::None,
                                        name: poly_name_interned,
                                        name_span: poly_span,
                                        has_default_impl: false,
                                        impls: vec![associated_item.name_span],
                                    });

                                    // push `#[poly] fn @associated_func_unwrap_1<T1, T2>(x: T1) -> T2;` to the session.
                                    self.new_funcs.push(Func {
                                        is_pure,
                                        impure_keyword_span: None,

                                        // TODO: I'm not sure whether it should be private/public
                                        //       I'll know that when I implement the visibility checker.
                                        visibility: Visibility::private(),

                                        keyword_span: Span::None,
                                        name: poly_name_interned,
                                        name_span: poly_span,
                                        generics: (0..(params + 1)).map(
                                            |i| Generic {
                                                name: generic_params[i],
                                                name_span: Span::Poly {
                                                    name: poly_name_interned,
                                                    kind: if i == params {
                                                        PolySpanKind::Return
                                                    } else {
                                                        PolySpanKind::Param(i)
                                                    },
                                                },
                                            },
                                        ).collect(),
                                        params: (0..params).map(
                                            |i| FuncParam {
                                                name: param_names[i],
                                                name_span: Span::None,
                                                type_annot: Some(Type::Path(Path {
                                                    id: IdentWithOrigin {
                                                        id: generic_params[i],
                                                        span: Span::None,
                                                        def_span: Span::Poly {
                                                            name: poly_name_interned,
                                                            kind: PolySpanKind::Param(i),
                                                        },
                                                        origin: NameOrigin::GenericParam { index: i },
                                                    },
                                                    fields: vec![],
                                                    types: vec![None],
                                                })),
                                                default_value: None,
                                            }
                                        ).collect(),
                                        type_annot: Some(Type::Path(Path {
                                            id: IdentWithOrigin {
                                                id: generic_params[params],
                                                span: Span::None,
                                                def_span: Span::Poly {
                                                    name: poly_name_interned,
                                                    kind: PolySpanKind::Return,
                                                },
                                                origin: NameOrigin::GenericParam { index: params },
                                            },
                                            fields: vec![],
                                            types: vec![None],
                                        })),
                                        value: Expr::dummy(),
                                        origin: FuncOrigin::AssociatedFunc,
                                        built_in: false,
                                        foreign_names: HashMap::new(),
                                        use_counts: HashMap::new(),
                                    });
                                },
                            }
                        }

                        else {
                            struct_shape.associated_lets.insert(associated_item.name, associated_item.name_span);
                        }
                    }

                    else {
                        todo!()
                    }
                },
                Err(e) => {
                    self.errors.push(e);
                    has_error = true;
                    continue;
                },
            }
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }
}
