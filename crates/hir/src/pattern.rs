use crate::{Session, Type};
use sodigy_error::{Error, ErrorKind};
use sodigy_name_analysis::IdentWithOrigin;
use sodigy_number::InternedNumber;
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::InfixOp;

#[derive(Clone, Debug)]
pub struct Pattern {
    pub name: Option<InternedString>,
    pub name_span: Option<Span>,
    pub r#type: Option<Type>,
    pub kind: PatternKind,
}

#[derive(Clone, Debug)]
pub enum PatternKind {
    Identifier {
        id: InternedString,
        span: Span,
    },
    DollarIdentifier(IdentWithOrigin),
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
    Tuple {
        elements: Vec<Pattern>,
        dot_dot_span: Option<Span>,
        group_span: Span,
    },
    List {
        elements: Vec<Pattern>,
        dot_dot_span: Option<Span>,
        group_span: Span,
    },
    Range {
        lhs: Option<Box<Pattern>>,
        rhs: Option<Box<Pattern>>,
        op_span: Span,
        is_inclusive: bool,
    },
    InfixOp {
        op: InfixOp,
        lhs: Box<Pattern>,
        rhs: Box<Pattern>,
        op_span: Span,
    },
    Or {
        lhs: Box<Pattern>,
        rhs: Box<Pattern>,
        op_span: Span,
    },
    Wildcard(Span),
}

impl Pattern {
    pub fn from_ast(ast_pattern: &ast::Pattern, session: &mut Session) -> Result<Pattern, ()> {
        let mut has_error = false;
        let r#type = match ast_pattern.r#type.as_ref().map(|r#type| Type::from_ast(r#type, session)) {
            Some(Ok(r#type)) => Some(r#type),
            Some(Err(())) => {
                has_error = true;
                None
            },
            None => None,
        };
        let kind = match PatternKind::from_ast(&ast_pattern.kind, session) {
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
                r#type,
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
    pub fn from_ast(ast_pattern: &ast::PatternKind, session: &mut Session) -> Result<PatternKind, ()> {
        match ast_pattern {
            ast::PatternKind::Identifier { id, span } => Ok(PatternKind::Identifier { id: *id, span: *span }),
            ast::PatternKind::DollarIdentifier { id, span } => match session.find_origin_and_count_usage(*id) {
                Some((origin, def_span)) => {
                    Ok(PatternKind::DollarIdentifier(IdentWithOrigin {
                        id: *id,
                        span: *span,
                        origin,
                        def_span,
                    }))
                },
                None => {
                    session.errors.push(Error {
                        kind: ErrorKind::UndefinedName(*id),
                        spans: span.simple_error(),
                        note: None,
                    });
                    Err(())
                },
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
            ast::PatternKind::Tuple { elements: ast_elements, dot_dot_span, group_span } |
            ast::PatternKind::List { elements: ast_elements, dot_dot_span, group_span } => {
                let is_tuple = matches!(ast_pattern, ast::PatternKind::Tuple { .. });
                let mut has_error = false;
                let mut elements = Vec::with_capacity(ast_elements.len());

                for ast_element in ast_elements.iter() {
                    match Pattern::from_ast(ast_element, session) {
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
                    Ok(PatternKind::Tuple { elements, dot_dot_span: *dot_dot_span, group_span: *group_span })
                }

                else {
                    Ok(PatternKind::List { elements, dot_dot_span: *dot_dot_span, group_span: *group_span })
                }
            },
            ast::PatternKind::Range { lhs, rhs, op_span, is_inclusive } => match (
                lhs.as_ref().map(|lhs| Pattern::from_ast(lhs, session)),
                rhs.as_ref().map(|rhs| Pattern::from_ast(rhs, session)),
            ) {
                (Some(Err(())), _) | (_, Some(Err(()))) => Err(()),
                (lhs, rhs) => Ok(PatternKind::Range {
                    lhs: lhs.map(|lhs| Box::new(lhs.unwrap())),
                    rhs: rhs.map(|rhs| Box::new(rhs.unwrap())),
                    op_span: *op_span,
                    is_inclusive: *is_inclusive,
                }),
            },
            ast::PatternKind::InfixOp { op, lhs, rhs, op_span } => match (
                Pattern::from_ast(lhs, session),
                Pattern::from_ast(rhs, session),
            ) {
                (Err(()), _) | (_, Err(())) => Err(()),
                (lhs, rhs) => Ok(PatternKind::InfixOp {
                    op: *op,
                    lhs: Box::new(lhs.unwrap()),
                    rhs: Box::new(rhs.unwrap()),
                    op_span: *op_span,
                }),
            },
            ast::PatternKind::Or { lhs, rhs, op_span } => match (
                Pattern::from_ast(lhs, session),
                Pattern::from_ast(rhs, session),
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
            PatternKind::Wildcard(_) |
            PatternKind::DollarIdentifier { .. } => vec![],
            PatternKind::Identifier { id, span } => vec![(*id, *span)],
            _ => todo!(),
        }
    }
}
