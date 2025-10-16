// - FULL_PATTERN
//   - IDENT (COLON TYPE)
//   - IDENT (COLON TYPE) AT PATTERN
//   - PATTERN
// - PATTERN
//   - LITERAL
//   - REGEX
//   - IDENT (DOT IDENT)*
//   - DOLLAR INDENT
//   - WILDCARD
//   - IDENT (DOT IDENT)* STRUCT_PATTERN
//   - (IDENT (DOT IDENT)*)? TUPLE_PATTERN
//   - LIST_PATTERN
//   - OPEN_PAREN PATTERN CLOSE_PAREN
//   - PATTERN? (DOTDOT | DOTDOT_EQ) PATTERN
//   - PATTERN OR PATTERN
//     - You cannot bind name like `a @ 0 | b @ 1`, but you can do `a @ (0 | 1)` or `(a @ 0, 1) | (a @ 2, 3)`.
//   - FULL_PATTERN CONCAT FULL_PATTERN
// - TUPLE_PATTERN
//   - OPEN_PAREN CLOSE_PAREN
//   - OPEN_PAREN FULL_PATTERN (COMMA FULL_PATTERN)+ CLOSE_PAREN
// - LIST_PATTERN
//   - OPEN_BRACKET CLOSE_BRACKET
//   - OPEN_BRACKET FULL_PATTERN (COMMA FULL_PATTERN)* CLOSE_BRACKET
// - STRUCT_PATTERN
//   - OPEN_BRACE FIELD_PATTERN (COMMA FIELD_PATTERN)* CLOSE_BRACE
// - FIELD_PATTERN
//   - IDENT
//   - IDENT COLON PATTERN
//   - DOTDOT

use crate::{Tokens, Type};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Delim, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct FullPattern {
    pub name: Option<InternedString>,
    pub name_span: Option<Span>,
    pub r#type: Option<Type>,
    pub pattern: Pattern,
}

impl FullPattern {
    pub fn bound_names(&self) -> Vec<(InternedString, Span)> {
        let mut result = vec![];

        if let (Some(name), Some(name_span)) = (self.name, self.name_span) {
            result.push((name, name_span));
        }

        result.extend(self.pattern.bound_names());
        result
    }
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
    Path(Vec<(InternedString, Span)>),
    Wildcard(Span),
    Struct {
        r#struct: Vec<(InternedString, Span)>,
        fields: Vec<StructFieldPattern>,
        dot_dot: bool,
        group_span: Span,
    },
    TupleStruct {
        r#struct: Vec<(InternedString, Span)>,
        elements: Vec<FullPattern>,
        group_span: Span,
    },
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
    Or {
        lhs: Box<Pattern>,
        rhs: Box<Pattern>,
        op_span: Span,
    },
    Concat {
        lhs: Box<FullPattern>,
        rhs: Box<FullPattern>,
        op_span: Span,
    },
}

#[derive(Clone, Debug)]
pub struct StructFieldPattern {
    pub name: InternedString,
    pub span: Span,
    pub pattern: FullPattern,
}

impl Pattern {
    pub fn error_span(&self) -> Span {
        match self {
            Pattern::Number { span, .. } |
            Pattern::Identifier { span, .. } |
            Pattern::Wildcard(span) |
            Pattern::Tuple { group_span: span, .. } |
            Pattern::List { group_span: span, .. } |
            Pattern::Range { op_span: span, .. } |
            Pattern::Or { op_span: span, .. } |
            Pattern::Concat { op_span: span, .. } => *span,
            Pattern::Path(path) |
            Pattern::Struct { r#struct: path, .. } |
            Pattern::TupleStruct { r#struct: path, .. } => {
                let mut result = path[0].1;

                for (_, span) in path.iter() {
                    result = result.merge(*span);
                }

                result
            },
        }
    }

    pub fn bound_names(&self) -> Vec<(InternedString, Span)> {
        match self {
            Pattern::Number { .. } |
            Pattern::Wildcard(_) |
            Pattern::Path(_) => vec![],
            Pattern::Identifier { id, span } => vec![(*id, *span)],
            Pattern::Struct { fields, .. } => fields.iter().flat_map(|f| f.pattern.bound_names()).collect(),
            Pattern::TupleStruct { elements, .. } |
            Pattern::Tuple { elements, .. } |
            Pattern::List { elements, .. } => elements.iter().flat_map(|e| e.bound_names()).collect(),
            Pattern::Range { lhs, rhs, .. } => {
                let mut result = vec![];

                if let Some(lhs) = lhs {
                    result.extend(lhs.bound_names());
                }

                if let Some(rhs) = rhs {
                    result.extend(rhs.bound_names());
                }

                result
            },
            Pattern::Or { lhs, rhs, .. } => vec![
                lhs.bound_names(),
                rhs.bound_names(),
            ].concat(),
            Pattern::Concat { lhs, rhs, .. } => vec![
                lhs.bound_names(),
                rhs.bound_names(),
            ].concat(),
        }
    }
}

impl<'t> Tokens<'t> {
    // It only does necessary checks. All the other checks are done by `FullPattern::check()`.
    pub fn parse_full_pattern(&mut self) -> Result<FullPattern, Vec<Error>> {
        let mut name = None;
        let mut name_span = None;
        let mut r#type = None;

        match self.peek2() {
            (
                Some(Token { kind: TokenKind::Identifier(id), span }),
                Some(Token { kind: TokenKind::Punct(Punct::Colon), .. }),
            ) => {
                name = Some(*id);
                name_span = Some(*span);
                self.cursor += 2;
                r#type = Some(self.parse_type()?);

                match self.peek() {
                    Some(Token { kind: TokenKind::Punct(Punct::At), .. }) => {
                        self.cursor += 1;
                    },
                    _ => {
                        return Ok(FullPattern {
                            name,
                            name_span,
                            r#type,

                            // It treats `x: Int` like `x: Int @ _`.
                            pattern: Pattern::Wildcard(Span::None),
                        });
                    },
                }
            },
            (
                Some(Token { kind: TokenKind::Identifier(id), span }),
                Some(Token { kind: TokenKind::Punct(Punct::At), .. }),
            ) => {
                name = Some(*id);
                name_span = Some(*span);
                self.cursor += 2;
            },
            _ => {},
        }

        let pattern = self.parse_pattern()?;
        let lhs = FullPattern {
            name,
            name_span,
            r#type,
            pattern,
        };

        match self.peek() {
            Some(Token { kind: TokenKind::Punct(Punct::Concat), span }) => {
                let op_span = *span;
                self.cursor += 1;
                let rhs = self.parse_full_pattern()?;

                // How can we bind a name to a concat pattern?
                Ok(FullPattern {
                    name: None,
                    name_span: None,
                    r#type: None,
                    pattern: Pattern::Concat {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        op_span,
                    },
                })
            },
            _ => Ok(lhs),
        }
    }

    pub fn parse_full_patterns(&mut self) -> Result<Vec<FullPattern>, Vec<Error>> {
        if self.is_empty() {
            return Ok(vec![]);
        }

        let mut patterns = vec![];

        loop {
            patterns.push(self.parse_full_pattern()?);

            match self.peek2() {
                (
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }),
                    None,
                ) => {
                    self.cursor += 1;
                    return Ok(patterns);
                },
                (
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }),
                    Some(_),
                ) => {
                    self.cursor += 1;
                },
                (Some(_), _) => {},
                (None, _) => {
                    return Ok(patterns);
                },
            }
        }
    }

    pub fn parse_pattern(&mut self) -> Result<Pattern, Vec<Error>> {
        let lhs = match self.peek2() {
            (
                Some(Token { kind: TokenKind::Identifier(id), span }),
                Some(Token { kind: TokenKind::Punct(Punct::Dot), .. }),
            ) => todo!(),
            (
                Some(Token { kind: TokenKind::Identifier(id), span: span1 }),
                Some(Token { kind: TokenKind::Group { delim, tokens }, span: span2 }),
            ) => match *delim {
                Delim::Parenthesis => todo!(),
                Delim::Brace => todo!(),
                Delim::Bracket | Delim::Lambda => {
                    return Err(vec![Error {
                        kind: ErrorKind::UnexpectedToken {
                            expected: ErrorToken::ParenthesisOrBrace,
                            got: ErrorToken::Group(*delim),
                        },
                        span: *span2,
                        ..Error::default()
                    }]);
                },
            },
            (Some(Token { kind: TokenKind::Identifier(id), span }), _) => {
                let (id, span) = (*id, *span);
                self.cursor += 1;

                match id.try_unintern_short_string() {
                    Some(id) if id == b"_" => Pattern::Wildcard(span),
                    _ => Pattern::Identifier { id, span },
                }
            },
            (Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, span }), _) => {
                let span = *span;
                let mut tokens = Tokens::new(tokens, span.end());
                let elements = tokens.parse_full_patterns()?;
                let mut is_tuple = elements.len() != 1;

                // `(a)` is not a tuple pattern, it's just a name binding
                if elements.len() == 1 && matches!(
                    tokens.last(),
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }),
                ) {
                    is_tuple = true;
                }

                self.cursor += 1;

                if is_tuple {
                    Pattern::Tuple {
                        elements,
                        group_span: span,
                    }
                }

                else {
                    let mut errors = vec![];

                    if let (Some(name), Some(name_span)) = (elements[0].name, elements[0].name_span) {
                        errors.push(Error {
                            kind: ErrorKind::CannotBindNameToAnotherName(name),
                            span: name_span,
                            ..Error::default()
                        });
                    }

                    if let Some(r#type) = &elements[0].r#type {
                        errors.push(Error {
                            kind: ErrorKind::CannotAnnotateType,
                            span: r#type.error_span(),
                            ..Error::default()
                        });
                    }

                    if errors.is_empty() {
                        elements[0].pattern.clone()
                    }

                    else {
                        return Err(errors);
                    }
                }
            },
            (Some(Token { kind: TokenKind::Group { delim: Delim::Bracket, tokens }, span }), _) => {
                let span = *span;
                let mut tokens = Tokens::new(tokens, span.end());
                let elements = tokens.parse_full_patterns()?;
                self.cursor += 1;
                Pattern::List {
                    elements,
                    group_span: span,
                }
            },
            (Some(Token { kind: TokenKind::Number(n), span }), _) => {
                let (n, span) = (*n, *span);
                self.cursor += 1;
                Pattern::Number { n, span }
            },
            // `..3`
            (Some(Token { kind: TokenKind::Punct(p @ (Punct::DotDot | Punct::DotDotEq)), span }), _) => {
                let op_span = *span;
                let is_inclusive = matches!(p, Punct::DotDotEq);
                self.cursor += 1;

                let rhs = self.parse_pattern()?;
                Pattern::Range {
                    lhs: None,
                    rhs: Some(Box::new(rhs)),
                    op_span,
                    is_inclusive,
                }
            },
            (t1, t2) => panic!("TODO: ({t1:?}, {t2:?})"),
        };

        match self.peek() {
            Some(Token { kind: TokenKind::Punct(Punct::Or), span }) => {
                let op_span = *span;
                self.cursor += 1;

                let rhs = self.parse_pattern()?;
                Ok(Pattern::Or {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    op_span,
                })
            },
            Some(Token { kind: TokenKind::Punct(p @ (Punct::DotDot | Punct::DotDotEq)), span }) => {
                let op_span = *span;
                let is_inclusive = matches!(p, Punct::DotDotEq);
                self.cursor += 1;

                if let Some(true) = self.peek().map(|t| t.pattern_begin()) {
                    let rhs = self.parse_pattern()?;

                    Ok(Pattern::Range {
                        lhs: Some(Box::new(lhs)),
                        rhs: Some(Box::new(rhs)),
                        op_span,
                        is_inclusive,
                    })
                }

                else {
                    Ok(Pattern::Range {
                        lhs: Some(Box::new(lhs)),
                        rhs: None,
                        op_span,
                        is_inclusive,
                    })
                }
            },
            _ => Ok(lhs),
        }
    }
}
