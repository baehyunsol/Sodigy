use crate::{Expr, Path, Session, eval_const};
use sodigy_error::Error;
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_parse::{self as ast, RestPattern};
use sodigy_span::{Span, SpanDeriveKind};
use sodigy_string::{InternedString, intern_string, unintern_string};
use sodigy_token::{Constant, InfixOp};

mod from_expr;
mod or;

pub use or::{PatternSplit, unreachable_or_pattern};

#[derive(Clone, Debug)]
pub struct Pattern {
    pub name: Option<InternedString>,
    pub name_span: Option<Span>,
    pub kind: PatternKind,
}

#[derive(Clone, Debug)]
pub enum PatternKind {
    Path(Path),
    Constant(Constant),
    NameBinding {
        id: InternedString,
        span: Span,
    },
    Regex {
        s: InternedString,
        span: Span,
    },
    Struct {
        r#struct: Path,
        fields: Vec<StructFieldPattern>,
        rest: Option<Span>,
        group_span: Span,
    },
    TupleStruct {
        r#struct: Path,
        elements: Vec<Pattern>,
        rest: Option<Box<RestPattern>>,
        group_span: Span,
    },
    Tuple {
        elements: Vec<Pattern>,
        rest: Option<Box<RestPattern>>,
        group_span: Span,
    },
    List {
        elements: Vec<Pattern>,
        rest: Option<Box<RestPattern>>,
        group_span: Span,
    },
    Range {
        lhs: Option<Box<Pattern>>,
        rhs: Option<Box<Pattern>>,
        op_span: Span,
        is_inclusive: bool,
    },
    Or {
        lhs: Box<Pattern>,
        rhs: Box<Pattern>,
        op_span: Span,
    },
    Wildcard(Span),
}

#[derive(Clone, Debug)]
pub struct StructFieldPattern {
    pub name: InternedString,
    pub span: Span,
    pub pattern: Pattern,
    pub is_shorthand: bool,
}

#[derive(Clone, Debug)]
pub struct ExtraGuard {
    pub name: InternedString,
    pub span: Span,
    pub condition: Expr,
}

impl Pattern {
    pub fn from_ast(
        ast_pattern: &ast::Pattern,
        session: &mut Session,
        extra_guards: &mut Vec<ExtraGuard>,
    ) -> Result<Pattern, ()> {
        let mut has_error = false;
        let kind = match PatternKind::from_ast(&ast_pattern.kind, session, extra_guards) {
            Ok(kind) => Some(kind),
            Err(()) => {
                has_error = true;
                None
            },
        };

        if has_error {
            Err(())
        }

        else {
            Ok(Pattern {
                name: ast_pattern.name,
                name_span: ast_pattern.name_span.clone(),
                kind: kind.unwrap(),
            })
        }
    }

    pub fn bound_names(&self) -> Vec<(InternedString, Span)> {
        let mut result = vec![];

        if let (Some(name), Some(name_span)) = (self.name, &self.name_span) {
            result.push((name, name_span.clone()));
        }

        result.extend(self.kind.bound_names());
        result
    }

    pub fn error_span_narrow(&self) -> Span {
        self.kind.error_span_narrow()
    }

    pub fn error_span_wide(&self) -> Span {
        if let Some(name_span) = &self.name_span {
            name_span.merge(&self.kind.error_span_wide())
        }

        else {
            self.kind.error_span_wide()
        }
    }
}

impl PatternKind {
    pub fn from_ast(
        ast_pattern: &ast::PatternKind,
        session: &mut Session,
        extra_guards: &mut Vec<ExtraGuard>,
    ) -> Result<PatternKind, ()> {
        match ast_pattern {
            // If `x` is an expression, `Some(x)` is lowered to `Some($tmp) if tmp == x`.
            // If `x` is an enum variant, `Some(x)` is lowered to `Some(x)`.
            // But the problem is that we don't know whether `x` is an expression or not
            // until inter-hir is complete. So we do the lowering later.
            ast::PatternKind::Path(p) => Ok(PatternKind::Path(Path::from_ast(p, session)?)),
            ast::PatternKind::Constant(Constant::String { binary, s, span }) => {
                // FIXME: Too many unwraps...
                let s = unintern_string(*s, &session.intermediate_dir).unwrap().unwrap();

                let elements = if *binary {
                    s.into_iter().map(
                        |b| Pattern {
                            name: None,
                            name_span: None,
                            kind: PatternKind::Constant(Constant::Byte { b, span: Span::None })
                        }
                    ).collect()
                } else {
                    let s = String::from_utf8(s).unwrap();
                    s.chars().map(
                        |ch| Pattern {
                            name: None,
                            name_span: None,
                            kind: PatternKind::Constant(Constant::Char { ch: ch as u32, span: Span::None })
                        }
                    ).collect()
                };

                Ok(PatternKind::List {
                    elements,
                    rest: None,
                    group_span: span.clone(),
                })
            },
            ast::PatternKind::Constant(c) => Ok(PatternKind::Constant(c.clone())),
            ast::PatternKind::NameBinding { id, span } => Ok(PatternKind::NameBinding { id: *id, span: span.clone() }),
            ast::PatternKind::Regex { s, span } => {
                session.errors.push(Error::todo(
                    18211,
                    "regex pattern",
                    span.clone(),
                ));
                Err(())
            },
            ast::PatternKind::Struct { r#struct, fields: ast_fields, rest, group_span } => {
                let mut has_error = false;
                let mut fields = Vec::with_capacity(ast_fields.len());
                let r#struct = match Path::from_ast(r#struct, session) {
                    Ok(path) => Some(path),
                    Err(()) => {
                        has_error = true;
                        None
                    },
                };

                for ast_field in ast_fields.iter() {
                    match StructFieldPattern::from_ast(ast_field, session, extra_guards) {
                        Ok(field) => {
                            fields.push(field);
                        },
                        Err(()) => {
                            has_error = true;
                        },
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(PatternKind::Struct {
                        r#struct: r#struct.unwrap(),
                        fields,
                        rest: rest.clone(),
                        group_span: group_span.clone(),
                    })
                }
            },
            ast::PatternKind::TupleStruct { r#struct, elements: ast_elements, rest, group_span } => {
                let mut has_error = false;
                let mut elements = Vec::with_capacity(ast_elements.len());
                let r#struct = match Path::from_ast(r#struct, session) {
                    Ok(path) => Some(path),
                    Err(()) => {
                        has_error = true;
                        None
                    },
                };

                for ast_element in ast_elements.iter() {
                    match Pattern::from_ast(ast_element, session, extra_guards) {
                        Ok(pattern) => {
                            elements.push(pattern);
                        },
                        Err(()) => {
                            has_error = true;
                        },
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(PatternKind::TupleStruct {
                        r#struct: r#struct.unwrap(),
                        elements,
                        rest: rest.clone(),
                        group_span: group_span.clone(),
                    })
                }
            },
            ast::PatternKind::Tuple { elements: ast_elements, rest, group_span } |
            ast::PatternKind::List { elements: ast_elements, rest, group_span, .. } => {
                let is_tuple = matches!(ast_pattern, ast::PatternKind::Tuple { .. });
                let mut has_error = false;
                let mut elements = Vec::with_capacity(ast_elements.len());

                for ast_element in ast_elements.iter() {
                    match Pattern::from_ast(ast_element, session, extra_guards) {
                        Ok(pattern) => {
                            elements.push(pattern);
                        },
                        Err(()) => {
                            has_error = true;
                        },
                    }
                }

                if has_error {
                    Err(())
                }

                else if is_tuple {
                    Ok(PatternKind::Tuple { elements, rest: rest.clone(), group_span: group_span.clone() })
                }

                else {
                    Ok(PatternKind::List { elements, rest: rest.clone(), group_span: group_span.clone() })
                }
            },
            ast::PatternKind::Range { lhs, rhs, op_span, is_inclusive } => match (
                lhs.as_ref().map(|lhs| Pattern::from_ast(lhs, session, extra_guards)),
                rhs.as_ref().map(|rhs| Pattern::from_ast(rhs, session, extra_guards)),
            ) {
                (Some(Err(())), _) | (_, Some(Err(()))) => Err(()),
                (lhs, rhs) => Ok(PatternKind::Range {
                    lhs: lhs.map(|lhs| Box::new(lhs.unwrap())),
                    rhs: rhs.map(|rhs| Box::new(rhs.unwrap())),
                    op_span: op_span.clone(),
                    is_inclusive: *is_inclusive,
                }),
            },
            ast::PatternKind::InfixOp { kind, lhs, op_span, rhs, .. } => {
                let result_span = lhs.error_span_wide().merge(op_span).merge(&rhs.error_span_wide());

                match kind {
                    ast::PatternValueKind::Constant | ast::PatternValueKind::Value => {
                        let ast_expr = match ast::Expr::from_pattern_kind(ast_pattern) {
                            Ok(expr) => expr,
                            Err(e) => {
                                session.errors.extend(e);
                                return Err(());
                            },
                        };
                        let expr = Expr::from_ast(&ast_expr, session)?;

                        match kind {
                            ast::PatternValueKind::Constant => {
                                let e = eval_const(&expr, session)?;
                                Ok(PatternKind::from_expr(&e, session)?)
                            },
                            // `Some(x + 1)` is lowered to `Some($tmp) if tmp == x + 1`
                            ast::PatternValueKind::Value => {
                                let tmp_value_name = intern_string(b"$tmp", &session.intermediate_dir).unwrap();
                                let derived_span = result_span.derive(SpanDeriveKind::ExprInPattern);
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
                                    rhs: Box::new(expr),
                                    op_span: derived_span.clone(),
                                };
                                extra_guards.push(ExtraGuard {
                                    name: tmp_value_name,
                                    span: derived_span.clone(),
                                    condition: extra_guard,
                                });
                                Ok(PatternKind::NameBinding { id: tmp_value_name, span: derived_span.clone() })
                            },
                            _ => unreachable!(),
                        }
                    },
                    ast::PatternValueKind::NameBinding => todo!(),  // turn this into an NameBinding, and encode the offset somewhere
                }
            },
            ast::PatternKind::Or { lhs, rhs, op_span } => match (
                Pattern::from_ast(lhs, session, extra_guards),
                Pattern::from_ast(rhs, session, extra_guards),
            ) {
                (Ok(lhs), Ok(rhs)) => {
                    // `1 | _` is lowered to `_`.
                    // It's necessary for post-mir.
                    match (&lhs.kind, &rhs.kind) {
                        // If both are wildcard, `lhs` is matched.
                        (PatternKind::Wildcard(_) | PatternKind::NameBinding { .. }, _) => {
                            session.warnings.push(unreachable_or_pattern(&lhs, &rhs));
                            Ok(lhs.kind)
                        },
                        (_, PatternKind::Wildcard(_) | PatternKind::NameBinding { .. }) => {
                            session.warnings.push(unreachable_or_pattern(&rhs, &lhs));
                            Ok(rhs.kind)
                        },
                        _ => Ok(PatternKind::Or {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                            op_span: op_span.clone(),
                        }),
                    }
                },
                _ => Err(()),
            },
            ast::PatternKind::Wildcard(span) => Ok(PatternKind::Wildcard(span.clone())),
            _ => panic!("TODO: {ast_pattern:?}"),
        }
    }

    pub fn bound_names(&self) -> Vec<(InternedString, Span)> {
        match self {
            PatternKind::Path(_) |
            PatternKind::Constant(_) |
            PatternKind::Regex { .. } |
            PatternKind::Wildcard(_) => vec![],
            PatternKind::NameBinding { id, span } => vec![(*id, span.clone())],
            PatternKind::TupleStruct { elements, rest, .. } |
            PatternKind::Tuple { elements, rest, .. } |
            PatternKind::List { elements, rest, .. } => {
                let mut result = elements.iter().flat_map(|e| e.bound_names()).collect::<Vec<_>>();

                if let Some(rest) = rest {
                    if let (Some(name), Some(name_span)) = (rest.name, &rest.name_span) {
                        result.push((name, name_span.clone()));
                    }
                }

                result
            },
            _ => todo!(),
        }
    }

    pub fn error_span_narrow(&self) -> Span {
        match self {
            PatternKind::Path(p) => p.error_span_narrow(),
            PatternKind::Constant(c) => c.span(),
            PatternKind::NameBinding { span, .. } |
            PatternKind::Regex { span, .. } |
            PatternKind::Wildcard(span) |
            PatternKind::Tuple { group_span: span, .. } |
            PatternKind::List { group_span: span, .. } |
            PatternKind::Range { op_span: span, .. } |
            PatternKind::Or { op_span: span, .. } => span.clone(),
            _ => panic!("TODO: {self:?}"),
        }
    }

    pub fn error_span_wide(&self) -> Span {
        match self {
            PatternKind::Path(p) => p.error_span_wide(),
            PatternKind::Constant(c) => c.span(),
            PatternKind::NameBinding { span, .. } |
            PatternKind::Regex { span, .. } |
            PatternKind::Wildcard(span) |
            PatternKind::Tuple { group_span: span, .. } |
            PatternKind::List { group_span: span, .. } => span.clone(),
            PatternKind::Range { lhs, op_span, rhs, .. } => {
                let mut span = match lhs {
                    Some(lhs) => lhs.error_span_wide().merge(op_span),
                    None => op_span.clone(),
                };

                if let Some(rhs) = rhs {
                    span = span.merge(&rhs.error_span_wide());
                }

                span
            },
            PatternKind::Or { lhs, rhs, op_span } => lhs.error_span_wide()
                .merge(op_span)
                .merge(&rhs.error_span_wide()),
            _ => panic!("TODO: {self:?}"),
        }
    }
}

impl StructFieldPattern {
    pub fn from_ast(
        ast_field: &ast::StructFieldPattern,
        session: &mut Session,
        extra_guards: &mut Vec<ExtraGuard>,
    ) -> Result<StructFieldPattern, ()> {
        Ok(StructFieldPattern {
            name: ast_field.name.clone(),
            span: ast_field.span.clone(),
            pattern: Pattern::from_ast(&ast_field.pattern, session, extra_guards)?,
            is_shorthand: ast_field.is_shorthand,
        })
    }
}
