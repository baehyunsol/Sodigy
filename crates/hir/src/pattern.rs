use crate::{Expr, Session, Type, eval_const};
use sodigy_error::{Error, ErrorKind};
use sodigy_name_analysis::{IdentWithOrigin, NameKind, NameOrigin};
use sodigy_number::InternedNumber;
use sodigy_parse::{self as ast, RestPattern};
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};
use sodigy_token::InfixOp;

mod from_expr;

#[derive(Clone, Debug)]
pub struct Pattern {
    pub name: Option<InternedString>,
    pub name_span: Option<Span>,
    pub kind: PatternKind,
}

#[derive(Clone, Debug)]
pub enum PatternKind {
    Ident {
        id: InternedString,
        span: Span,
    },
    Number {
        n: InternedNumber,
        span: Span,
    },
    String {
        binary: bool,
        s: InternedString,
        span: Span,
    },
    Regex {
        s: InternedString,
        span: Span,
    },
    Char {
        ch: u32,
        span: Span,
    },
    Byte {
        b: u8,
        span: Span,
    },
    Path(Vec<(InternedString, Span)>),
    Struct {
        r#struct: Vec<(InternedString, Span)>,
        fields: Vec<StructFieldPattern>,
        rest: Option<RestPattern>,
        group_span: Span,
    },
    TupleStruct {
        r#struct: Vec<(InternedString, Span)>,
        elements: Vec<Pattern>,
        rest: Option<RestPattern>,
        group_span: Span,
    },
    Tuple {
        elements: Vec<Pattern>,
        rest: Option<RestPattern>,
        group_span: Span,
    },
    List {
        elements: Vec<Pattern>,
        rest: Option<RestPattern>,
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

impl Pattern {
    pub fn from_ast(
        ast_pattern: &ast::Pattern,
        session: &mut Session,
        extra_guards: &mut Vec<(InternedString, Span, Expr)>,
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
                name_span: ast_pattern.name_span,
                kind: kind.unwrap(),
            })
        }
    }

    pub fn bound_names(&self) -> Vec<(InternedString, Span)> {
        let mut result = vec![];

        if let (Some(name), Some(name_span)) = (self.name, self.name_span) {
            result.push((name, name_span));
        }

        result.extend(self.kind.bound_names());
        result
    }
}

impl PatternKind {
    pub fn from_ast(
        ast_pattern: &ast::PatternKind,
        session: &mut Session,
        extra_guards: &mut Vec<(InternedString, Span, Expr)>,
    ) -> Result<PatternKind, ()> {
        match ast_pattern {
            ast::PatternKind::Ident { id, span } => Ok(PatternKind::Ident { id: *id, span: *span }),
            // `Some($x)` is lowered to `Some(tmp) if tmp == x`
            ast::PatternKind::DollarIdent { id, span } => {
                let ref_id = match session.find_origin_and_count_usage(*id) {
                    Some((origin, def_span)) => IdentWithOrigin {
                        id: *id,
                        span: *span,
                        origin,
                        def_span,
                    },
                    None => {
                        session.errors.push(Error {
                            kind: ErrorKind::UndefinedName(*id),
                            spans: span.simple_error(),
                            note: None,
                        });
                        return Err(());
                    },
                };
                let tmp_name = intern_string(b"$tmp", &session.intermediate_dir).unwrap();
                let tmp_span = Span::None;  // TODO: impl derived span
                let extra_guard = Expr::InfixOp {
                    op: InfixOp::Eq,
                    lhs: Box::new(Expr::Ident(IdentWithOrigin {
                        id: tmp_name,
                        span: tmp_span,
                        origin: NameOrigin::Local { kind: NameKind::PatternNameBind },
                        def_span: Span::None,  // TODO: impl derived span
                    })),
                    rhs: Box::new(Expr::Ident(ref_id)),
                    op_span: Span::None,  // TODO: impl derived span
                };
                extra_guards.push((tmp_name, tmp_span, extra_guard));
                Ok(PatternKind::Ident { id: tmp_name, span: tmp_span })
            },
            ast::PatternKind::Number { n, span } => Ok(PatternKind::Number { n: n.clone(), span: *span }),
            ast::PatternKind::String { binary, s, span } => Ok(PatternKind::String { binary: *binary, s: *s, span: *span }),
            ast::PatternKind::Regex { s, span } => {
                session.errors.push(Error::todo(
                    18211,
                    "regex pattern",
                    *span,
                ));
                Err(())
            },
            ast::PatternKind::Char { ch, span } => Ok(PatternKind::Char { ch: *ch, span: *span }),
            ast::PatternKind::Byte { b, span } => Ok(PatternKind::Byte { b: *b, span: *span }),
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
                    Ok(PatternKind::Tuple { elements, rest: *rest, group_span: *group_span })
                }

                else {
                    Ok(PatternKind::List { elements, rest: *rest, group_span: *group_span })
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
                    op_span: *op_span,
                    is_inclusive: *is_inclusive,
                }),
            },
            ast::PatternKind::InfixOp { kind, .. } => match kind {
                ast::PatternValueKind::Constant | ast::PatternValueKind::DollarIdent => {
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
                        ast::PatternValueKind::DollarIdent => {
                            let tmp_name = intern_string(b"$tmp", &session.intermediate_dir).unwrap();
                            let tmp_span = Span::None;  // TODO: impl derived span
                            let extra_guard = Expr::InfixOp {
                                op: InfixOp::Eq,
                                lhs: Box::new(Expr::Ident(IdentWithOrigin {
                                    id: tmp_name,
                                    span: tmp_span,
                                    origin: NameOrigin::Local { kind: NameKind::PatternNameBind },
                                    def_span: Span::None,  // TODO: impl derived span
                                })),
                                rhs: Box::new(expr),
                                op_span: Span::None,  // TODO: impl derived span
                            };
                            extra_guards.push((tmp_name, tmp_span, extra_guard));
                            Ok(PatternKind::Ident { id: tmp_name, span: tmp_span })
                        },
                        _ => unreachable!(),
                    }
                },
                ast::PatternValueKind::Ident => todo!(),  // turn this into an Ident, and encode the offset somewhere
            },
            ast::PatternKind::Or { lhs, rhs, op_span } => match (
                Pattern::from_ast(lhs, session, extra_guards),
                Pattern::from_ast(rhs, session, extra_guards),
            ) {
                (Err(()), _) | (_, Err(())) => Err(()),
                (lhs, rhs) => Ok(PatternKind::Or {
                    lhs: Box::new(lhs.unwrap()),
                    rhs: Box::new(rhs.unwrap()),
                    op_span: *op_span,
                }),
            },
            ast::PatternKind::Wildcard(span) => Ok(PatternKind::Wildcard(*span)),
            _ => panic!("TODO: {ast_pattern:?}"),
        }
    }

    pub fn bound_names(&self) -> Vec<(InternedString, Span)> {
        match self {
            PatternKind::Number { .. } |
            PatternKind::String { .. } |
            PatternKind::Regex { .. } |
            PatternKind::Char { .. } |
            PatternKind::Byte { .. } |
            PatternKind::Path(_) |
            PatternKind::Wildcard(_) => vec![],
            PatternKind::Ident { id, span } => vec![(*id, *span)],
            PatternKind::TupleStruct { elements, .. } |
            PatternKind::Tuple { elements, .. } |
            PatternKind::List { elements, .. } => elements.iter().flat_map(|e| e.bound_names()).collect(),
            _ => todo!(),
        }
    }
}
