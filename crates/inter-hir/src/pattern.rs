use crate::{Session, TypeStructExpr, not_x_but_y};
use sodigy_error::{EnumFieldKind, Error, ErrorKind, NotXBut};
use sodigy_hir::{
    EnumVariant,
    Expr,
    ExtraGuard,
    Path,
    Pattern,
    PatternKind,
};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_span::SpanDeriveKind;
use sodigy_string::intern_string;
use sodigy_token::InfixOp;

impl Session {
    pub fn resolve_pattern(&mut self, pattern: &mut Pattern) -> Result<(), ()> {
        let mut has_error = false;

        if let Err(()) = self.resolve_pattern_kind(&mut pattern.kind) {
            has_error = true;
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    pub fn resolve_pattern_kind(&mut self, kind: &mut PatternKind) -> Result<(), ()> {
        match kind {
            PatternKind::Path(path) => self.resolve_path(path, None, &mut vec![]),
            PatternKind::Constant(_) |
            PatternKind::NameBinding { .. } |
            PatternKind::Regex { .. } |
            PatternKind::Wildcard(_) => Ok(()),
            PatternKind::Struct { r#struct, fields, .. } => {
                let mut has_error = self.resolve_path(r#struct, None, &mut vec![]).is_err();

                for field in fields.iter_mut() {
                    if let Err(()) = self.resolve_pattern(&mut field.pattern) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            PatternKind::TupleStruct { r#struct, elements, .. } => {
                let mut has_error = self.resolve_path(r#struct, None, &mut vec![]).is_err();

                for element in elements.iter_mut() {
                    if let Err(()) = self.resolve_pattern(element) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            PatternKind::Tuple { elements, .. } |
            PatternKind::List { elements, .. } => {
                let mut has_error = false;

                for element in elements.iter_mut() {
                    if let Err(()) = self.resolve_pattern(element) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            PatternKind::Range { lhs, rhs, .. } => {
                let mut has_error = false;

                if let Some(lhs) = lhs {
                    if let Err(()) = self.resolve_pattern(lhs) {
                        has_error = true;
                    }
                }

                if let Some(rhs) = rhs {
                    if let Err(()) = self.resolve_pattern(rhs) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            PatternKind::Or { lhs, rhs, .. } => match (
                self.resolve_pattern(lhs),
                self.resolve_pattern(rhs),
            ) {
                (Ok(()), Ok(())) => Ok(()),
                _ => Err(()),
            },
        }
    }

    // It lowers `Some(x)` to `Some($tmp) if tmp == x`.
    pub fn check_pattern_path(
        &mut self,
        pattern: &mut Pattern,
        extra_guards: &mut Vec<ExtraGuard>,
    ) -> Result<(), ()> {
        self.check_pattern_kind_path(&mut pattern.kind, extra_guards)
    }

    pub fn check_pattern_kind_path(
        &mut self,
        pattern_kind: &mut PatternKind,
        extra_guards: &mut Vec<ExtraGuard>,
    ) -> Result<(), ()> {
        match pattern_kind {
            PatternKind::Path(p) => {
                match &p.id.origin {
                    NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => match kind {
                        NameKind::EnumVariant => {
                            let enum_def_span = self.variant_to_enum_span.get(&p.id.def_span).unwrap();
                            let enum_shape = self.enum_shapes.get(enum_def_span).unwrap();
                            let variant: &EnumVariant = &enum_shape.variants[*enum_shape.variant_index.get(&p.id.def_span).unwrap()];

                            match EnumFieldKind::from(&variant.fields) {
                                EnumFieldKind::None => {
                                    return Ok(());
                                },
                                e => {
                                    self.errors.push(Error {
                                        kind: ErrorKind::MismatchedEnumFieldKind {
                                            expected: e,
                                            got: EnumFieldKind::None,
                                        },
                                        spans: p.id.span.simple_error(),
                                        note: None,
                                    });
                                    return Err(());
                                },
                            }
                        },
                        NameKind::Struct => {
                            let mut e = not_x_but_y(p, TypeStructExpr::Expr, NotXBut::Struct, &self.intermediate_dir);
                            e.note = Some(String::from("The struct's fields are missing."));
                            self.errors.push(e);
                            return Err(());
                        },
                        _ => {},
                    },
                    _ => {},
                }

                let tmp_value_name = intern_string(b"$tmp", &self.intermediate_dir).unwrap();
                let derived_span = p.id.span.derive(SpanDeriveKind::ExprInPattern);
                let extra_guard = Expr::InfixOp {
                    op: InfixOp::Eq,
                    lhs: Box::new(Expr::Path(Path {
                        id: IdentWithOrigin {
                            id: tmp_value_name,
                            span: derived_span.clone(),
                            origin: NameOrigin::Local { kind: NameKind::PatternNameBind },
                            def_span: derived_span.clone(),
                        },
                        fields: vec![],
                        dotfish: vec![None],
                    })),
                    rhs: Box::new(Expr::Path(p.clone())),
                    op_span: derived_span.clone(),
                };

                *pattern_kind = PatternKind::NameBinding { id: tmp_value_name, span: derived_span.clone() };
                extra_guards.push(ExtraGuard {
                    name: tmp_value_name,
                    span: derived_span.clone(),
                    condition: extra_guard,
                });
                Ok(())
            },
            PatternKind::Constant(_) |
            PatternKind::NameBinding { .. } |
            PatternKind::Regex { .. } |
            PatternKind::Wildcard(_) => Ok(()),
            PatternKind::Struct { r#struct, fields, .. } => {
                let mut has_error = false;

                if let Err(()) = self.check_struct_path(r#struct, EnumFieldKind::Struct) {
                    has_error = true;
                }

                for field in fields.iter_mut() {
                    if let Err(()) = self.check_pattern_path(&mut field.pattern, extra_guards) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            PatternKind::TupleStruct { r#struct, elements, .. } => {
                let mut has_error = false;

                if let Err(()) = self.check_struct_path(r#struct, EnumFieldKind::Tuple) {
                    has_error = true;
                }

                for element in elements.iter_mut() {
                    if let Err(()) = self.check_pattern_path(element, extra_guards) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            PatternKind::Tuple { elements, .. } |
            PatternKind::List { elements, .. } => {
                let mut has_error = false;

                for element in elements.iter_mut() {
                    if let Err(()) = self.check_pattern_path(element, extra_guards) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            PatternKind::Range { lhs, rhs, .. } => {
                let mut has_error = false;

                if let Some(lhs) = lhs && let Err(()) = self.check_pattern_path(lhs, extra_guards) {
                    has_error = true;
                }

                if let Some(rhs) = rhs && let Err(()) = self.check_pattern_path(rhs, extra_guards) {
                    has_error = true;
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            PatternKind::Or { lhs, rhs, .. } => {
                let mut has_error = false;

                if let Err(()) = self.check_pattern_path(lhs, extra_guards) {
                    has_error = true;
                }

                if let Err(()) = self.check_pattern_path(rhs, extra_guards) {
                    has_error = true;
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
        }
    }

    fn check_struct_path(&mut self, path: &Path, field_kind: EnumFieldKind) -> Result<(), ()> {
        match &path.id.origin {
            NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => match kind {
                NameKind::EnumVariant => {
                    let enum_def_span = self.variant_to_enum_span.get(&path.id.def_span).unwrap();
                    let enum_shape = self.enum_shapes.get(enum_def_span).unwrap();
                    let variant: &EnumVariant = &enum_shape.variants[*enum_shape.variant_index.get(&path.id.def_span).unwrap()];

                    match (EnumFieldKind::from(&variant.fields), field_kind) {
                        (EnumFieldKind::Struct, EnumFieldKind::Struct) => Ok(()),
                        (EnumFieldKind::Tuple, EnumFieldKind::Tuple) => Ok(()),
                        (expected, got) => {
                            self.errors.push(Error {
                                kind: ErrorKind::MismatchedEnumFieldKind { expected, got },
                                spans: path.id.span.simple_error(),
                                note: None,
                            });
                            Err(())
                        },
                    }
                },
                NameKind::Struct => match field_kind {
                    EnumFieldKind::Struct => Ok(()),
                    EnumFieldKind::Tuple => {
                        self.errors.push(not_x_but_y(path, TypeStructExpr::TupleStruct, NotXBut::Struct, &self.intermediate_dir));
                        Err(())
                    },
                    _ => unreachable!(),
                },
                _ => {
                    self.errors.push(not_x_but_y(path, field_kind.into(), kind.into(), &self.intermediate_dir));
                    Err(())
                },
            },
            _ => {
                let kind = match &path.id.origin {
                    NameOrigin::FuncParam { .. } => NotXBut::Expr,
                    NameOrigin::GenericParam { .. } => NotXBut::GenericParam,
                    _ => unreachable!(),
                };

                self.errors.push(not_x_but_y(path, field_kind.into(), kind, &self.intermediate_dir));
                Err(())
            },
        }
    }
}
