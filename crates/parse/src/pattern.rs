use crate::{Tokens, Type};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_number::InternedNumber;
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use sodigy_token::{Delim, InfixOp, Punct, Token, TokenKind};

mod eval;
use eval::eval_const_pattern;

#[derive(Clone, Debug)]
pub struct Pattern {
    // `name` and `name_span` are for extra name bindings, like `x @ 0..10`.
    // So, `PatternKind::Ident` doesn't have these fields.
    pub name: Option<InternedString>,
    pub name_span: Option<Span>,

    // You can add a type annotation only if its kind is `PatternKind::Ident`.
    pub r#type: Option<Type>,

    pub kind: PatternKind,
}

#[derive(Clone, Debug)]
pub enum PatternKind {
    // A name binding.
    // `if let x = foo() { .. }`
    Ident {
        id: InternedString,
        span: Span,
    },
    // Capturing a name.
    // `let x = 3; if let $x = foo() { .. }` matches if `foo()` is `3`.
    DollarIdent {
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
        dot_dot_span: Option<Span>,
        group_span: Span,
    },
    TupleStruct {
        r#struct: Vec<(InternedString, Span)>,
        elements: Vec<Pattern>,
        dot_dot_span: Option<Span>,
        group_span: Span,
    },
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
    // `1..10`
    // `1..=9`
    // `1..`
    // `..10`
    // `..` -> it's invalid
    Range {
        lhs: Option<Box<Pattern>>,
        rhs: Option<Box<Pattern>>,
        op_span: Span,
        is_inclusive: bool,
    },
    // `if let Some(x + 1) = foo() { x }`
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

#[derive(Clone, Debug)]
pub struct StructFieldPattern {
    pub name: InternedString,
    pub span: Span,
    pub pattern: Pattern,
    pub is_shorthand: bool,
}

impl Pattern {
    pub fn bound_names(&self) -> Vec<(InternedString, Span)> {
        let mut result = vec![];

        if let (Some(name), Some(name_span)) = (self.name, self.name_span) {
            result.push((name, name_span));
        }

        result.extend(self.kind.bound_names());
        result
    }

    pub fn error_span(&self) -> Span {
        if let Some(name_span) = self.name_span {
            name_span.merge(self.kind.error_span())
        }

        else {
            self.kind.error_span()
        }
    }
}

impl PatternKind {
    pub fn error_span(&self) -> Span {
        match self {
            PatternKind::Number { span, .. } |
            PatternKind::String { span, .. } |
            PatternKind::Regex { span, .. } |
            PatternKind::Char { span, .. } |
            PatternKind::Byte { span, .. } |
            PatternKind::Ident { span, .. } |
            PatternKind::Wildcard(span) |
            PatternKind::DollarIdent { span, .. } |
            PatternKind::Tuple { group_span: span, .. } |
            PatternKind::List { group_span: span, .. } |
            PatternKind::Range { op_span: span, .. } |
            PatternKind::Or { op_span: span, .. } |
            PatternKind::InfixOp { op_span: span, .. } => *span,
            PatternKind::Path(path) |
            PatternKind::Struct { r#struct: path, .. } |
            PatternKind::TupleStruct { r#struct: path, .. } => {
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
            PatternKind::Number { .. } |
            PatternKind::String { .. } |
            PatternKind::Regex { .. } |
            PatternKind::Char { .. } |
            PatternKind::Byte { .. } |
            PatternKind::Path(_) |
            PatternKind::Wildcard(_) |
            PatternKind::DollarIdent { .. } => vec![],
            PatternKind::Ident { id, span } => vec![(*id, *span)],
            PatternKind::Struct { fields, .. } => fields.iter().flat_map(|f| f.pattern.bound_names()).collect(),
            PatternKind::TupleStruct { elements, .. } |
            PatternKind::Tuple { elements, .. } |
            PatternKind::List { elements, .. } => elements.iter().flat_map(|e| e.bound_names()).collect(),
            PatternKind::Range { lhs, rhs, .. } => {
                let mut result = vec![];

                if let Some(lhs) = lhs {
                    result.extend(lhs.bound_names());
                }

                if let Some(rhs) = rhs {
                    result.extend(rhs.bound_names());
                }

                result
            },
            // NOTE: `lhs` and `rhs` of `Pattern::Or` must have the exact
            //       same name bindings, otherwise a compile error. But
            //       checking the compile error is not a responsibility of
            //       this function, and it just assumes that there's no error.
            PatternKind::Or { lhs, .. } => lhs.bound_names(),
            PatternKind::InfixOp { lhs, rhs, .. } => vec![
                lhs.bound_names(),
                rhs.bound_names(),
            ].concat(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ParsePatternContext {
    MatchArm,
    IfLet,
    Let,

    // Inside parenthesis or brackets
    Group,
}

impl<'t> Tokens<'t> {
    pub fn parse_pattern(&mut self, context: ParsePatternContext) -> Result<Pattern, Vec<Error>> {
        self.pratt_parse_pattern(context, 0)
    }

    // It only catches errors that make it im-parse-able.
    // All the other checks are done by `Pattern::check()`.
    fn pratt_parse_pattern(&mut self, context: ParsePatternContext, min_bp: u32) -> Result<Pattern, Vec<Error>> {
        let mut lhs = match self.peek2() {
            (
                Some(Token { kind: TokenKind::Ident(id), span }),
                Some(Token { kind: TokenKind::Punct(Punct::Colon), .. }),
            ) => {
                let (id, span) = (*id, *span);
                self.cursor += 2;
                let r#type = self.parse_type()?;

                Pattern {
                    name: None,
                    name_span: None,
                    r#type: Some(r#type),
                    kind: PatternKind::Ident { id, span },
                }
            },
            (
                Some(Token { kind: TokenKind::Ident(id), span }),
                Some(Token { kind: TokenKind::Punct(Punct::Dot), .. }),
            ) => todo!(),
            (Some(Token { kind: TokenKind::Ident(id), span }), _) => {
                let (id, span) = (*id, *span);
                self.cursor += 1;

                if &id.try_unintern_short_string().unwrap_or(vec![]) == b"_" {
                    Pattern {
                        name: None,
                        name_span: None,
                        r#type: None,
                        kind: PatternKind::Wildcard(span),
                    }
                }

                else {
                    Pattern {
                        name: None,
                        name_span: None,
                        r#type: None,
                        kind: PatternKind::Ident { id, span },
                    }
                }
            },
            (
                Some(Token { kind: TokenKind::Punct(Punct::Dollar), .. }),
                Some(Token { kind: TokenKind::Ident(id), span }),
            ) => {
                let (id, span) = (*id, *span);
                self.cursor += 2;
                Pattern {
                    name: None,
                    name_span: None,
                    r#type: None,
                    kind: PatternKind::Ident { id, span },
                }
            },
            (
                Some(Token { kind: TokenKind::Punct(Punct::Sub), span: op_span }),
                Some(Token { kind: TokenKind::Number(n), span: n_span }),
            ) => {
                let (mut n, op_span, n_span) = (n.clone(), *op_span, *n_span);
                n.negate_mut();
                let span = op_span.merge(n_span);
                self.cursor += 2;
                Pattern {
                    name: None,
                    name_span: None,
                    r#type: None,
                    kind: PatternKind::Number { n, span },
                }
            },
            (Some(Token { kind: TokenKind::Number(n), span }), _) => {
                let (n, span) = (n.clone(), *span);
                self.cursor += 1;
                Pattern {
                    name: None,
                    name_span: None,
                    r#type: None,
                    kind: PatternKind::Number { n, span },
                }
            },
            (Some(Token { kind: TokenKind::String { binary, raw: _, regex, s }, span }), _) => {
                let (binary, regex, s, span) = (*binary, *regex, *s, *span);
                self.cursor += 1;

                if regex {
                    Pattern {
                        name: None,
                        name_span: None,
                        r#type: None,
                        kind: PatternKind::Regex { s, span },
                    }
                }

                else {
                    Pattern {
                        name: None,
                        name_span: None,
                        r#type: None,
                        kind: PatternKind::String { binary, s, span },
                    }
                }
            },
            (Some(Token { kind: TokenKind::Char(ch), span }), _) => {
                let (ch, span) = (*ch, *span);
                self.cursor += 1;
                Pattern {
                    name: None,
                    name_span: None,
                    r#type: None,
                    kind: PatternKind::Char { ch, span },
                }
            },
            (Some(Token { kind: TokenKind::Byte(b), span }), _) => {
                let (b, span) = (*b, *span);
                self.cursor += 1;
                Pattern {
                    name: None,
                    name_span: None,
                    r#type: None,
                    kind: PatternKind::Byte { b, span },
                }
            },
            (Some(Token { kind: TokenKind::Group { delim, tokens }, span }), _) => {
                match delim {
                    Delim::Brace |
                    Delim::Lambda |
                    Delim::Decorator |
                    Delim::ModuleDecorator => {
                        return Err(vec![Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: ErrorToken::BraceOrParenthesis,
                                got: ErrorToken::Group(*delim),
                            },
                            spans: span.simple_error(),
                            note: None,
                        }]);
                    },
                    Delim::Parenthesis |
                    Delim::Bracket => {},
                }

                let (group_span, delim) = (*span, *delim);
                let mut tokens = Tokens::new(tokens, span.end());
                let (mut elements, dot_dot_span) = tokens.parse_patterns(/* may_have_dot_dot: */ true)?;

                // If it's parenthesis, we have to distinguish `(3)` and `(3,)`
                let mut is_tuple = elements.len() != 1 || dot_dot_span.is_some();

                if !is_tuple && matches!(
                    tokens.last(),
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }),
                ) {
                    is_tuple = true;
                }

                self.cursor += 1;

                match delim {
                    Delim::Parenthesis => if is_tuple {
                        Pattern {
                            name: None,
                            name_span: None,
                            r#type: None,
                            kind: PatternKind::Tuple {
                                elements,
                                group_span,
                                dot_dot_span,
                            },
                        }
                    } else {
                        elements.remove(0)
                    },
                    Delim::Bracket => Pattern {
                        name: None,
                        name_span: None,
                        r#type: None,
                        kind: PatternKind::List {
                            elements,
                            group_span,
                            dot_dot_span,
                        },
                    },
                    Delim::Brace |
                    Delim::Lambda |
                    Delim::Decorator |
                    Delim::ModuleDecorator => unreachable!(),
                }
            },
            (Some(Token { kind: TokenKind::Punct(p @ (Punct::DotDot | Punct::DotDotEq)), span }), _) => {
                let is_inclusive = matches!(p, Punct::DotDotEq);
                let op_span = *span;
                self.cursor += 1;
                let rhs = self.parse_pattern(context)?;

                Pattern {
                    name: None,
                    name_span: None,
                    r#type: None,
                    kind: PatternKind::Range {
                        lhs: None,
                        rhs: Some(Box::new(rhs)),
                        op_span,
                        is_inclusive,
                    },
                }
            },
            ts => panic!("TODO: {ts:?}"),
        };

        loop {
            match self.peek() {
                Some(Token { kind: TokenKind::Punct(p), span }) => {
                    let (p, op_span) = (*p, *span);

                    match p {
                        Punct::At => {
                            let (l_bp, r_bp) = name_binding_binding_power();

                            if l_bp < min_bp {
                                break;
                            }

                            self.cursor += 1;

                            let mut rhs = self.pratt_parse_pattern(context, r_bp)?;
                            let mut name_binding = None;
                            let mut errors = vec![];

                            match lhs {
                                Pattern {
                                    name,
                                    name_span,
                                    r#type,
                                    kind: PatternKind::Ident { id, span },
                                } => {
                                    name_binding = Some((id, span));

                                    // `a @ b @ 1..2`
                                    if let (Some(name), Some(name_span)) = (name, name_span) {
                                        errors.push(Error {
                                            kind: ErrorKind::RedundantNameBinding(name, id),
                                            spans: vec![
                                                RenderableSpan {
                                                    span: name_span,
                                                    auxiliary: false,
                                                    note: None,
                                                },
                                                RenderableSpan {
                                                    span,
                                                    auxiliary: false,
                                                    note: None,
                                                },
                                            ],
                                            note: None,
                                        });
                                    }

                                    if let Some(r#type) = r#type {
                                        errors.push(Error {
                                            kind: ErrorKind::CannotAnnotateType,
                                            spans: vec![
                                                RenderableSpan {
                                                    span: lhs.kind.error_span(),
                                                    auxiliary: true,
                                                    note: Some(String::from("You cannot add type annotation to this.")),
                                                },
                                                RenderableSpan {
                                                    span: r#type.error_span(),
                                                    auxiliary: false,
                                                    note: Some(String::from("Remove this type annotation.")),
                                                },
                                            ],
                                            note: None,
                                        });
                                    }
                                },
                                _ => {
                                    let expected_token = match context {
                                        ParsePatternContext::MatchArm => ErrorToken::Punct(Punct::Arrow),
                                        ParsePatternContext::IfLet | ParsePatternContext::Let => ErrorToken::Punct(Punct::Assign),
                                        ParsePatternContext::Group => ErrorToken::Punct(Punct::Comma),
                                    };

                                    return Err(vec![Error {
                                        kind: ErrorKind::UnexpectedToken {
                                            expected: expected_token,
                                            got: ErrorToken::Punct(Punct::At),
                                        },
                                        spans: op_span.simple_error(),
                                        note: None,
                                    }]);
                                },
                            }

                            match &rhs {
                                // `a @ b @ 1..2`
                                Pattern { name: Some(name), name_span: Some(name_span), .. } |
                                // `a @ b`
                                Pattern { kind: PatternKind::Ident { id: name, span: name_span }, .. } => {
                                    let (prev_name, prev_name_span) = name_binding.unwrap();
                                    errors.push(Error {
                                        kind: ErrorKind::RedundantNameBinding(*name, prev_name),
                                        spans: vec![
                                            RenderableSpan {
                                                span: *name_span,
                                                auxiliary: false,
                                                note: Some(String::from("It binds a name, so you don't have to bind again.")),
                                            },
                                            RenderableSpan {
                                                span: prev_name_span,
                                                auxiliary: false,
                                                note: None,
                                            },
                                        ],
                                        note: None,
                                    });
                                },
                                _ => {
                                    let (name, name_span) = name_binding.unwrap();
                                    rhs.name = Some(name);
                                    rhs.name_span = Some(name_span);
                                },
                            }

                            if !errors.is_empty() {
                                return Err(errors);
                            }

                            lhs = rhs;
                        },
                        Punct::Or => {
                            let (l_bp, r_bp) = or_binding_power();

                            if l_bp < min_bp {
                                break;
                            }

                            self.cursor += 1;
                            let rhs = self.pratt_parse_pattern(context, r_bp)?;
                            lhs = Pattern {
                                name: None,
                                name_span: None,
                                r#type: None,
                                kind: PatternKind::Or {
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                    op_span,
                                },
                            };
                        },
                        Punct::DotDot | Punct::DotDotEq => {
                            let (l_bp, r_bp) = range_binding_power();
                            let is_inclusive = matches!(p, Punct::DotDotEq);

                            if l_bp < min_bp {
                                break;
                            }

                            self.cursor += 1;

                            match self.peek().map(|t| t.pattern_begin()) {
                                Some(true) => {
                                    let rhs = self.pratt_parse_pattern(context, r_bp)?;
                                    lhs = Pattern {
                                        name: None,
                                        name_span: None,
                                        r#type: None,
                                        kind: PatternKind::Range {
                                            lhs: Some(Box::new(lhs)),
                                            rhs: Some(Box::new(rhs)),
                                            op_span,
                                            is_inclusive,
                                        },
                                    };
                                },
                                _ => {
                                    lhs = Pattern {
                                        name: None,
                                        name_span: None,
                                        r#type: None,
                                        kind: PatternKind::Range {
                                            lhs: Some(Box::new(lhs)),
                                            rhs: None,
                                            op_span,
                                            is_inclusive,
                                        },
                                    };
                                },
                            }
                        },
                        p => match InfixOp::try_from(p) {
                            Ok(op) => {
                                let (l_bp, r_bp) = match infix_binding_power(op) {
                                    Some((l, r)) => (l, r),
                                    None => {
                                        let expected_token = match context {
                                            ParsePatternContext::MatchArm => ErrorToken::Punct(Punct::Arrow),
                                            ParsePatternContext::IfLet | ParsePatternContext::Let => ErrorToken::Punct(Punct::Assign),
                                            ParsePatternContext::Group => ErrorToken::Punct(Punct::Comma),
                                        };

                                        return Err(vec![Error {
                                            kind: ErrorKind::UnexpectedToken {
                                                expected: expected_token,
                                                got: ErrorToken::Punct(p),
                                            },
                                            spans: op_span.simple_error(),
                                            note: None,
                                        }]);
                                    },
                                };

                                if l_bp < min_bp {
                                    break;
                                }

                                self.cursor += 1;
                                let rhs = self.pratt_parse_pattern(context, r_bp)?;
                                let kind = eval_const_pattern(op, lhs, rhs, op_span)?;
                                lhs = Pattern {
                                    name: None,
                                    name_span: None,
                                    r#type: None,
                                    kind,
                                };
                                continue;
                            },
                            Err(_) => {
                                // Okay, `p` is not an operator. we should not touch this.
                                break;
                            },
                        },
                    }
                },
                Some(t) => panic!("TODO: {t:?}"),
                None => {
                    break;
                },
            }
        }

        Ok(lhs)
    }

    pub fn parse_patterns(&mut self, may_have_dot_dot: bool) -> Result<(Vec<Pattern>, Option<Span>), Vec<Error>> {
        let mut prev_dot_dot_span = None;
        let mut patterns = vec![];

        loop {
            match self.peek2() {
                (
                    Some(Token { kind: TokenKind::Punct(Punct::DotDot), span }),
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }) | None,
                ) => {
                    if !may_have_dot_dot {
                        return Err(vec![Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: ErrorToken::Pattern,
                                got: ErrorToken::Punct(Punct::DotDot),
                            },
                            spans: span.simple_error(),
                            note: Some(String::from("You cannot use a rest pattern (`..`) here. If you intended a range without ends, that's still invalid. A range must have at least one end, otherwise, just use a wildcard pattern.")),
                        }]);
                    }

                    else if let Some(prev_dot_dot_span) = prev_dot_dot_span {
                        return Err(vec![Error {
                            kind: ErrorKind::MultipleDotDotsInPattern,
                            spans: vec![
                                RenderableSpan {
                                    span: *span,
                                    auxiliary: false,
                                    note: None,
                                },
                                RenderableSpan {
                                    span: prev_dot_dot_span,
                                    auxiliary: true,
                                    note: None,
                                },
                            ],
                            note: None,
                        }]);
                    }

                    else {
                        prev_dot_dot_span = Some(*span);
                        self.cursor += 2;
                    }
                },
                (Some(_), _) => {
                    patterns.push(self.parse_pattern(ParsePatternContext::Group)?);

                    match self.peek2() {
                        (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), None) | (None, _) => {
                            break;
                        },
                        (Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }), _) => {
                            self.cursor += 1;
                        },
                        (Some(t), _) => {
                            return Err(vec![Error {
                                kind: ErrorKind::UnexpectedToken {
                                    expected: ErrorToken::CommaOrGt,
                                    got: (&t.kind).into(),
                                },
                                spans: t.span.simple_error(),
                                note: None,
                            }]);
                        },
                    }
                },
                (None, _) => {
                    break;
                },
            }
        }

        Ok((patterns, prev_dot_dot_span))
    }
}

fn infix_binding_power(op: InfixOp) -> Option<(u32, u32)> {
    match op {
        InfixOp::Mul | InfixOp::Div | InfixOp::Rem => Some((MUL, MUL + 1)),
        InfixOp::Add | InfixOp::Sub => Some((ADD, ADD + 1)),
        InfixOp::Shl | InfixOp::Shr => Some((SHIFT, SHIFT + 1)),
        InfixOp::Concat => Some((CONCAT, CONCAT + 1)),
        _ => None,
    }
}

fn name_binding_binding_power() -> (u32, u32) {
    (NAME_BINDING, NAME_BINDING + 1)
}

fn or_binding_power() -> (u32, u32) {
    (OR, OR + 1)
}

fn range_binding_power() -> (u32, u32) {
    (RANGE, RANGE + 1)
}

const RANGE: u32 = 29;
const NAME_BINDING: u32 = 27;
const MUL: u32 = 25;  // a * b, a / b, a % b
const ADD: u32 = 23;  // a + b, a - b
const SHIFT: u32 = 21;  // a << b, a >> b
const CONCAT: u32 = 19;
const OR: u32 = 17;
