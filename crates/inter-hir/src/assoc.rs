use crate::Session;
use sodigy_error::{Error, ErrorKind};
use sodigy_hir::{
    AssociatedFunc,
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
use sodigy_string::{InternedString, intern_string};
use std::collections::hash_map::{Entry, HashMap};

impl Session {
    pub fn resolve_associated_items(&mut self) -> Result<(), ()> {
        fn get_def_span(associated_item: &AssociatedItem, r#type: &Type) -> Result<Span, Error> {
            match r#type {
                Type::Path(path) | Type::Param { constructor: path, .. } => {
                    match &path.id.origin {
                        NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => match kind {
                            NameKind::Struct => Ok(path.id.def_span.clone()),
                            NameKind::Enum => Ok(path.id.def_span.clone()),
                            NameKind::GenericParam => Err(Error {
                                kind: ErrorKind::TooGeneralToAssociateItem,
                                spans: associated_item.type_span.simple_error(),
                                note: None,
                            }),

                            // already filtered out by `check_type_annot_path`
                            _ => unreachable!(),
                        },

                        // already filtered out by `check_type_annot_path`
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

            else if let Err(()) = self.check_type_annot_path(&associated_item.r#type) {
                has_error = true;
                continue;
            }

            match get_def_span(&associated_item, &associated_item.r#type) {
                Ok(def_span) => {
                    let mut item_shape = self.get_item_shape(&def_span).unwrap();

                    for existing_association in item_shape.existing_associations() {
                        if existing_association.name == associated_item.name {
                            let error = match (existing_association.kind, associated_item.kind) {
                                (AssociatedItemKind::Func, AssociatedItemKind::Func) => {
                                    if associated_item.params == existing_association.params && associated_item.is_pure == existing_association.is_pure {
                                        // okay
                                        continue;
                                    }

                                    else {
                                        todo!()  // err
                                    }
                                },
                                (
                                    AssociatedItemKind::Field | AssociatedItemKind::Func,
                                    AssociatedItemKind::Field | AssociatedItemKind::Func,
                                ) => todo!(),  // err
                                (
                                    AssociatedItemKind::Variant | AssociatedItemKind::Let,
                                    AssociatedItemKind::Variant | AssociatedItemKind::Let,
                                ) => todo!(),  // err
                                _ => {
                                    // okay
                                    continue;
                                },
                            };

                            self.errors.push(error);
                            has_error = true;
                            continue 'associated_items;
                        }
                    }

                    if let AssociatedItemKind::Func = associated_item.kind {
                        let params = associated_item.params.unwrap();
                        let is_pure = associated_item.is_pure.unwrap();

                        match item_shape.associated_funcs_mut().entry(associated_item.name) {
                            Entry::Occupied(mut e) => {
                                e.get_mut().name_spans.push(associated_item.name_span.clone());
                            },
                            Entry::Vacant(e) => {
                                e.insert(AssociatedFunc {
                                    name: associated_item.name,
                                    name_spans: vec![associated_item.name_span.clone()],
                                    params,
                                    is_pure,
                                });
                            },
                        }

                        let poly_name = get_associated_func_name(associated_item.name, is_pure, params, &self.intermediate_dir);
                        let poly_name_interned = intern_string(poly_name.as_bytes(), &self.intermediate_dir).unwrap();
                        let poly_span: Span = Span::Poly {
                            name: poly_name_interned,
                            kind: PolySpanKind::Name,
                        };

                        match self.polys.entry(poly_span.clone()) {
                            Entry::Occupied(mut e) => {
                                e.get_mut().impls.push(associated_item.name_span.clone());
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
                                    name_span: poly_span.clone(),
                                    has_default_impl: false,
                                    impls: vec![associated_item.name_span.clone()],
                                });

                                for i in 0..(params + 1) {
                                    let poly_span_kind = if i == params {
                                        PolySpanKind::Return
                                    } else {
                                        PolySpanKind::Param(i)
                                    };

                                    self.generic_def_span_rev.insert(
                                        Span::Poly { name: poly_name_interned, kind: poly_span_kind },
                                        Span::Poly { name: poly_name_interned, kind: PolySpanKind::Name },
                                    );
                                }

                                // push `#[poly] fn @associated_func_unwrap_1<T1, T2>(x: T1) -> T2;` to the session.
                                let new_func = Func {
                                    is_pure,
                                    impure_keyword_span: None,

                                    // TODO: I'm not sure whether it should be private/public
                                    //       I'll know that when I implement the visibility checker.
                                    visibility: Visibility::private(),

                                    keyword_span: Span::None,
                                    name: poly_name_interned,
                                    name_span: poly_span.clone(),
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

                                    // 1. It's `Some(None)`, not `None`, because if `generics` is not empty, `generic_group_span` should not be `None`.
                                    // 2. We don't derive the span here because `generic_group_span` is only for error messages and the derived span doesn't
                                    //    help generating the error messages.
                                    generic_group_span: Some(Span::None),

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
                                                dotfish: vec![None],
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
                                        dotfish: vec![None],
                                    })),
                                    value: Expr::dummy(),
                                    origin: FuncOrigin::AssociatedFunc,
                                    built_in: false,
                                    foreign_names: HashMap::new(),
                                    captured_names: None,
                                    use_counts: HashMap::new(),
                                };
                                self.func_shapes.insert(new_func.name_span.clone(), new_func.shape());
                                self.new_funcs.push(new_func);
                            },
                        }
                    }

                    else {
                        item_shape.associated_lets_mut().insert(associated_item.name, associated_item.name_span.clone());
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

pub fn get_associated_func_name(
    name: InternedString,
    is_pure: bool,
    params: usize,
    intermediate_dir: &str,
) -> String {
    // Readability does not matter!! It tries hard to keep the name shorter than 16 bytes,
    // so that interner doesn't have to do file IO.
    let suffix = params * 2 + is_pure as usize;
    format!("{}:{suffix:x}", name.unintern_or_default(intermediate_dir))
}
