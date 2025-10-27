use crate::{Session, Type};
use sodigy_number::InternedNumber;
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct FullPattern {
    pub name: Option<InternedString>,
    pub name_span: Option<Span>,
    pub r#type: Option<Type>,
    pub pattern: Pattern,
}

#[derive(Clone, Debug)]
pub enum Pattern {
    Number {
        n: InternedNumber,
        span: Span,
    },
    Identifier {
        id: InternedString,
        span: Span,
    },
    Wildcard(Span),
    Tuple {
        elements: Vec<FullPattern>,
        group_span: Span,
    },
    List {
        elements: Vec<FullPattern>,
        group_span: Span,
    },
    Range {
        lhs: Option<Box<Pattern>>,
        rhs: Option<Box<Pattern>>,
        op_span: Span,
        is_inclusive: bool,
    },
}

impl FullPattern {
    pub fn from_ast(ast_pattern: &ast::FullPattern, session: &mut Session) -> Result<FullPattern, ()> {
        let mut has_error = false;
        let r#type = match ast_pattern.r#type.as_ref().map(|r#type| Type::from_ast(r#type, session)) {
            Some(Ok(r#type)) => Some(r#type),
            Some(Err(())) => {
                has_error = true;
                None
            },
            None => None,
        };
        let pattern = match Pattern::from_ast(&ast_pattern.pattern, session) {
            Ok(pattern) => Some(pattern),
            Err(()) => {
                has_error = true;
                None
            },
        };

        if has_error {
            Err(())
        }

        else {
            Ok(FullPattern {
                name: ast_pattern.name,
                name_span: ast_pattern.name_span,
                r#type,
                pattern: pattern.unwrap(),
            })
        }
    }
}

impl Pattern {
    pub fn from_ast(ast_pattern: &ast::Pattern, session: &mut Session) -> Result<Pattern, ()> {
        match ast_pattern {
            ast::Pattern::Number { n, span } => Ok(Pattern::Number { n: *n, span: *span }),
            ast::Pattern::Identifier { id, span } => Ok(Pattern::Identifier { id: *id, span: *span }),
            ast::Pattern::Wildcard(span) => Ok(Pattern::Wildcard(*span)),
            ast::Pattern::Tuple { elements: ast_elements, group_span } |
            ast::Pattern::List { elements: ast_elements, group_span } => {
                let is_tuple = matches!(ast_pattern, ast::Pattern::Tuple { .. });
                let mut has_error = false;
                let mut elements = Vec::with_capacity(ast_elements.len());

                for ast_element in ast_elements.iter() {
                    match FullPattern::from_ast(ast_element, session) {
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
                    Ok(Pattern::Tuple { elements, group_span: *group_span })
                }

                else {
                    Ok(Pattern::List { elements, group_span: *group_span })
                }
            },
            ast::Pattern::Range { lhs, rhs, op_span, is_inclusive } => match (
                lhs.as_ref().map(|lhs| Pattern::from_ast(lhs, session)),
                rhs.as_ref().map(|rhs| Pattern::from_ast(rhs, session)),
            ) {
                (Some(Err(())), _) | (_, Some(Err(()))) => Err(()),
                (lhs, rhs) => Ok(Pattern::Range {
                    lhs: lhs.map(|lhs| Box::new(lhs.unwrap())),
                    rhs: rhs.map(|rhs| Box::new(rhs.unwrap())),
                    op_span: *op_span,
                    is_inclusive: *is_inclusive,
                }),
            },
            _ => panic!("TODO: {ast_pattern:?}"),
        }
    }
}
