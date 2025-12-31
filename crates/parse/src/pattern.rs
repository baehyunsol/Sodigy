use crate::Tokens;
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_number::InternedNumber;
use sodigy_span::{RenderableSpan, Span, SpanDeriveKind};
use sodigy_string::{InternedString, unintern_string};
use sodigy_token::{Delim, InfixOp, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Pattern {
    // `name` and `name_span` are for extra name bindings, like `x @ 0..10`.
    // So, `PatternKind::Ident` doesn't have these fields.
    pub name: Option<InternedString>,
    pub name_span: Option<Span>,
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
        is_lowered_from_concat: bool,
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
    // It only cares about numeric operators.
    // `if let Some(x + 1) = foo() { x }`
    // `if let Some($x + $y) = foo() { 0 }`
    InfixOp {
        op: InfixOp,
        lhs: Box<Pattern>,
        rhs: Box<Pattern>,
        op_span: Span,
        kind: PatternValueKind,
    },
    Or {
        lhs: Box<Pattern>,
        rhs: Box<Pattern>,
        op_span: Span,
    },
    Wildcard(Span),

    // In `x |> match 3 { $ => 100, _ => 200 }`, `$` captures `x`.
    PipelineData(Span),
}

// By 'constant', it means Number/Char/Byte.
#[derive(Clone, Debug)]
pub enum PatternValueKind {
    // Every operand is a constant, like `Some((1 << 32) - 1)`.
    Constant,

    // Every operand is a constant or a dollar-ident, like `Some($x + $y + 1)`.
    DollarIdent,

    // Exactly one operand is an ident and the other operands are constants, like `Some(x + (1 << 32))`.
    Ident,
}

#[derive(Clone, Debug)]
pub struct StructFieldPattern {
    pub name: InternedString,
    pub span: Span,
    pub pattern: Pattern,
    pub is_shorthand: bool,
}

// `..` in `[a, b, .., c]`
// Parser guarantees that there's at most 1 rest in a group.
#[derive(Clone, Copy, Debug)]
pub struct RestPattern {
    pub span: Span,
    pub index: usize,

    // You can bind a name to dot_dot.
    // `[n] ++ ns` is lowered to `[n, ns @ ..]`.
    pub name: Option<InternedString>,
    pub name_span: Option<Span>,
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

    pub fn error_span_narrow(&self) -> Span {
        self.kind.error_span_narrow()
    }

    pub fn error_span_wide(&self) -> Span {
        if let Some(name_span) = self.name_span {
            name_span.merge(self.kind.error_span_wide())
        }

        else {
            self.kind.error_span_wide()
        }
    }

    // It's used to lower `[n] ++ ns` to `[n, ns @ ..]`.
    // Lhs and rhs of the concat operator are converted to a list pattern,
    // then their elements are concatonated.
    // - `ns` -> `[ns @ ..]`
    // - `[a, b, c]` -> `[a, b, c]`
    // - `"asdf"` -> `['a', 'b', 'c', 'd']`
    // - `[a] ++ [b]` -> `[a, b]`
    pub fn to_list_pattern(self, is_lhs: bool) -> Result<PatternKind, Vec<Error>> {
        let mut errors = vec![];

        if let (Some(name), Some(name_span)) = (self.name, self.name_span) {
            errors.push(Error {
                kind: ErrorKind::CannotBindName(name),
                spans: name_span.simple_error(),
                note: Some(String::from("I see what you're trying to do, and it's perfectly valid, but due to the limitations of the compiler, you cannot bind a name. A `++` pattern is lowered to a single long list, and there's no way to bind a name to a part of a list.")),
            });
        }

        let result = match self.kind {
            PatternKind::Ident { id, span } => PatternKind::List {
                elements: vec![],
                rest: Some(RestPattern {
                    span: span.derive(SpanDeriveKind::ConcatPatternRest),
                    index: 0,
                    name: Some(id),
                    name_span: Some(span),
                }),
                group_span: span.derive(SpanDeriveKind::ConcatPatternList),
                is_lowered_from_concat: true,
            },
            PatternKind::String { binary, s, span } => todo!(),
            l @ PatternKind::List { .. } => l,
            p => {
                errors.push(Error {
                    kind: ErrorKind::InvalidConcatPattern,
                    spans: p.error_span_wide().simple_error_with_note(
                        &format!("This cannot be an {} of `++`.", if is_lhs { "lhs" } else { "rhs" }),
                    ),
                    note: None,
                });
                return Err(errors);
            },
        };

        if errors.is_empty() {
            Ok(result)
        }

        else {
            Err(errors)
        }
    }
}

impl PatternKind {
    pub fn error_span_narrow(&self) -> Span {
        match self {
            PatternKind::Number { span, .. } |
            PatternKind::String { span, .. } |
            PatternKind::Regex { span, .. } |
            PatternKind::Char { span, .. } |
            PatternKind::Byte { span, .. } |
            PatternKind::Ident { span, .. } |
            PatternKind::Wildcard(span) |
            PatternKind::PipelineData(span) |
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

    pub fn error_span_wide(&self) -> Span {
        match self {
            _ => todo!(),
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
            PatternKind::PipelineData(_) |
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

impl<'t, 's> Tokens<'t, 's> {
    pub fn parse_pattern(&mut self, context: ParsePatternContext) -> Result<Pattern, Vec<Error>> {
        self.pratt_parse_pattern(context, 0)
    }

    // It only catches errors that make it im-parse-able.
    // All the other checks are done by `Pattern::check()`.
    fn pratt_parse_pattern(&mut self, context: ParsePatternContext, min_bp: u32) -> Result<Pattern, Vec<Error>> {
        let mut lhs = match self.peek2() {
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
                        kind: PatternKind::Wildcard(span),
                    }
                }

                else {
                    Pattern {
                        name: None,
                        name_span: None,
                        kind: PatternKind::Ident { id, span },
                    }
                }
            },
            (
                Some(Token { kind: TokenKind::Punct(Punct::Dollar), span: dollar_span }),
                Some(Token { kind: TokenKind::Ident(id), span: id_span }),
            ) => {
                let span = dollar_span.merge(*id_span);
                let id = *id;
                self.cursor += 2;
                Pattern {
                    name: None,
                    name_span: None,
                    kind: PatternKind::DollarIdent { id, span },
                }
            },
            (Some(Token { kind: TokenKind::Punct(Punct::Dollar), span }), _) => {
                let span = *span;
                self.cursor += 1;
                Pattern {
                    name: None,
                    name_span: None,
                    kind: PatternKind::PipelineData(span),
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
                    kind: PatternKind::Number { n, span },
                }
            },
            (Some(Token { kind: TokenKind::Number(n), span }), _) => {
                let (n, span) = (n.clone(), *span);
                self.cursor += 1;
                Pattern {
                    name: None,
                    name_span: None,
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
                        kind: PatternKind::Regex { s, span },
                    }
                }

                else {
                    Pattern {
                        name: None,
                        name_span: None,
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
                    kind: PatternKind::Char { ch, span },
                }
            },
            (Some(Token { kind: TokenKind::Byte(b), span }), _) => {
                let (b, span) = (*b, *span);
                self.cursor += 1;
                Pattern {
                    name: None,
                    name_span: None,
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
                    Delim::Parenthesis | Delim::Bracket => {},
                }

                let (group_span, delim) = (*span, *delim);
                let mut tokens = Tokens::new(tokens, span.end(), &self.intermediate_dir);
                let (mut elements, rest) = tokens.parse_patterns(/* may_have_dot_dot: */ true)?;

                // If it's parenthesis, we have to distinguish `(3)` and `(3,)`
                let mut is_tuple = elements.len() != 1 || rest.is_some();

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
                            kind: PatternKind::Tuple {
                                elements,
                                group_span,
                                rest,
                            },
                        }
                    } else {
                        elements.remove(0)
                    },
                    Delim::Bracket => Pattern {
                        name: None,
                        name_span: None,
                        kind: PatternKind::List {
                            elements,
                            group_span,
                            rest,
                            is_lowered_from_concat: false,
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
                            let name_binding;
                            let mut errors = vec![];

                            match lhs {
                                Pattern {
                                    name,
                                    name_span,
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
                                Pattern { kind: PatternKind::DollarIdent { id, span }, .. } => {
                                    let (name, name_span) = name_binding.unwrap();
                                    errors.push(Error {
                                        kind: ErrorKind::CannotBindName(name),
                                        spans: vec![
                                            RenderableSpan {
                                                span: name_span,
                                                auxiliary: false,
                                                note: None,
                                            },
                                            RenderableSpan {
                                                span: *span,
                                                auxiliary: false,
                                                note: Some(format!(
                                                    "It already has a name `{}`. You can just use this name.",
                                                    String::from_utf8_lossy(&unintern_string(*id, self.intermediate_dir).unwrap().unwrap_or(b"???".to_vec())),
                                                )),
                                            },
                                        ],
                                        note: None,
                                    });
                                },
                                Pattern { kind: PatternKind::InfixOp { .. }, .. } => {
                                    let (name, name_span) = name_binding.unwrap();
                                    errors.push(Error {
                                        kind: ErrorKind::CannotBindName(name),
                                        spans: name_span.simple_error(),
                                        note: Some(String::from("You cannot bind a name to a result of an infix operator. All infix operators in patterns are evaluated at compile time, and their intermediate results are gone.")),
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
                        // `[n] ++ ns` is immediately lowered to `[n, ns @ ..]`
                        Punct::Concat => {
                            let (l_bp, r_bp) = concat_binding_power();

                            if l_bp < min_bp {
                                break;
                            }

                            self.cursor += 1;
                            let rhs = self.pratt_parse_pattern(context, r_bp)?;

                            match (lhs.to_list_pattern(true), rhs.to_list_pattern(false)) {
                                (
                                    Ok(PatternKind::List {
                                        elements: elems1,
                                        rest: rest1,
                                        ..
                                    }),
                                    Ok(PatternKind::List {
                                        elements: elems2,
                                        rest: rest2,
                                        ..
                                    }),
                                ) => {
                                    let rest = match (rest1, rest2) {
                                        (Some(rest1), Some(rest2)) => {
                                            return Err(vec![Error {
                                                kind: ErrorKind::MultipleRestPatterns,
                                                spans: vec![
                                                    RenderableSpan {
                                                        span: rest1.span,
                                                        auxiliary: false,
                                                        note: Some(String::from("This matches an arbitrary number of elements.")),
                                                    },
                                                    RenderableSpan {
                                                        span: rest2.span,
                                                        auxiliary: false,
                                                        note: Some(String::from("This matches an arbitrary number of elements.")),
                                                    },
                                                ],
                                                note: None,
                                            }]);
                                        },
                                        (Some(rest), None) => Some(rest),
                                        (None, Some(mut rest)) => {
                                            // `[a, b, c] ++ [d, ..]` -> `[a, b, c, d, ..]`
                                            // `rest.index` is originally 1, but it has to be 4.
                                            rest.index += elems1.len();
                                            Some(rest)
                                        },
                                        (None, None) => None,
                                    };

                                    lhs = Pattern {
                                        name: None,
                                        name_span: None,
                                        kind: PatternKind::List {
                                            elements: vec![elems1, elems2].concat(),
                                            rest,
                                            group_span: op_span.derive(SpanDeriveKind::ConcatPatternList),
                                            is_lowered_from_concat: true,
                                        },
                                    };
                                },
                                (Ok(_), Ok(_)) => unreachable!(),
                                (Err(es1), Err(es2)) => {
                                    return Err(vec![es1, es2].concat());
                                },
                                (Err(e), _) => {
                                    return Err(e);
                                },
                                (_, Err(e)) => {
                                    return Err(e);
                                },
                            }
                        },
                        Punct::Eq => match context {
                            ParsePatternContext::Let | ParsePatternContext::IfLet => {
                                return Err(vec![Error {
                                    kind: ErrorKind::UnexpectedToken {
                                        expected: ErrorToken::Punct(Punct::Assign),
                                        got: ErrorToken::Punct(Punct::Eq),
                                    },
                                    spans: vec![
                                        RenderableSpan {
                                            span: op_span,
                                            auxiliary: false,
                                            note: Some(String::from("Use `=` instead of `==` here.")),
                                        },
                                    ],
                                    note: None,
                                }]);
                            },
                            _ => {
                                // Likely to be an error, another parser will catch this.
                                break;
                            },
                        },
                        Punct::ReturnType => match context {
                            ParsePatternContext::MatchArm => {
                                return Err(vec![Error {
                                    kind: ErrorKind::UnexpectedToken {
                                        expected: ErrorToken::Punct(Punct::Arrow),
                                        got: ErrorToken::Punct(Punct::ReturnType),
                                    },
                                    spans: vec![
                                        RenderableSpan {
                                            span: op_span,
                                            auxiliary: false,
                                            note: Some(String::from("Use `=>` instead of `->` here.")),
                                        },
                                    ],
                                    note: None,
                                }]);
                            },
                            _ => {
                                // Likely to be an error, another parser will catch this.
                                break;
                            },
                        },
                        p => match InfixOp::try_from(p) {
                            Ok(op) => {
                                let (l_bp, r_bp) = match infix_binding_power(op) {
                                    Some((l, r)) => (l, r),
                                    None => {
                                        return Err(vec![Error {
                                            kind: ErrorKind::UnsupportedInfixOpInPattern(op),
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
                                let op_kind = match (&lhs.kind, &rhs.kind) {
                                    (
                                        PatternKind::Number { .. } | PatternKind::Char { .. } | PatternKind::Byte { .. } | PatternKind::InfixOp { kind: PatternValueKind::Constant, .. },
                                        PatternKind::Number { .. } | PatternKind::Char { .. } | PatternKind::Byte { .. } | PatternKind::InfixOp { kind: PatternValueKind::Constant, .. },
                                    ) => PatternValueKind::Constant,
                                    (
                                        PatternKind::DollarIdent { .. } | PatternKind::Number { .. } | PatternKind::Char { .. } | PatternKind::Byte { .. } | PatternKind::InfixOp { kind: PatternValueKind::Constant | PatternValueKind::DollarIdent, .. },
                                        PatternKind::DollarIdent { .. } | PatternKind::Number { .. } | PatternKind::Char { .. } | PatternKind::Byte { .. } | PatternKind::InfixOp { kind: PatternValueKind::Constant | PatternValueKind::DollarIdent, .. },
                                    ) => PatternValueKind::DollarIdent,
                                    (
                                        PatternKind::Ident { .. } | PatternKind::InfixOp { kind: PatternValueKind::Ident, .. },
                                        PatternKind::Number { .. } | PatternKind::Char { .. } | PatternKind::Byte { .. } | PatternKind::InfixOp { kind: PatternValueKind::Constant, .. },
                                    ) | (
                                        PatternKind::Number { .. } | PatternKind::Char { .. } | PatternKind::Byte { .. } | PatternKind::InfixOp { kind: PatternValueKind::Constant, .. },
                                        PatternKind::Ident { .. } | PatternKind::InfixOp { kind: PatternValueKind::Ident, .. },
                                    ) => {
                                        let (aux_span, aux_name) = match (&lhs.kind, &rhs.kind) {
                                            (PatternKind::Ident { .. } | PatternKind::InfixOp { kind: PatternValueKind::Ident, .. }, _) => {
                                                (lhs.error_span_wide(), find_name_binding_in_const(&lhs.kind).unwrap())
                                            },
                                            (_, PatternKind::Ident { .. } | PatternKind::InfixOp { kind: PatternValueKind::Ident, .. }) => {
                                                (rhs.error_span_wide(), find_name_binding_in_const(&rhs.kind).unwrap())
                                            },
                                            _ => unreachable!(),
                                        };

                                        // `let x + 1 = 100;` is okay.
                                        // `let x * 2 = 100;` is not.
                                        match op {
                                            InfixOp::Add | InfixOp::Sub => {},
                                            _ => {
                                                return Err(vec![Error {
                                                    kind: ErrorKind::CannotApplyInfixOpToBinding,
                                                    spans: vec![
                                                        RenderableSpan {
                                                            span: op_span,
                                                            auxiliary: false,
                                                            note: Some(format!("`{}` is not allowed. Only `+` and `-` are allowed.", op.render_error())),
                                                        },
                                                        RenderableSpan {
                                                            span: aux_span,
                                                            auxiliary: true,
                                                            note: Some(format!(
                                                                "There's a name binding `{}`.",
                                                                String::from_utf8_lossy(&unintern_string(aux_name, self.intermediate_dir).unwrap().unwrap_or(b"???".to_vec())),
                                                            )),
                                                        },
                                                    ],
                                                    note: None,
                                                }]);
                                            },
                                        }

                                        PatternValueKind::Ident
                                    },
                                    (
                                        PatternKind::Ident { .. } | PatternKind::InfixOp { kind: PatternValueKind::Ident, .. },
                                        PatternKind::Ident { .. } | PatternKind::InfixOp { kind: PatternValueKind::Ident, .. },
                                    ) => {
                                        let lhs_name_binding = find_name_binding_in_const(&lhs.kind).unwrap();
                                        let rhs_name_binding = find_name_binding_in_const(&rhs.kind).unwrap();

                                        return Err(vec![Error {
                                            kind: ErrorKind::CannotApplyInfixOpToMultipleBindings,
                                            spans: vec![
                                                RenderableSpan {
                                                    span: op_span,
                                                    auxiliary: false,
                                                    note: None,
                                                },
                                                RenderableSpan {
                                                    span: lhs.error_span_wide(),
                                                    auxiliary: true,
                                                    note: Some(format!(
                                                        "There's a name binding `{}`.",
                                                        String::from_utf8_lossy(&unintern_string(lhs_name_binding, self.intermediate_dir).unwrap().unwrap_or(b"???".to_vec())),
                                                    )),
                                                },
                                                RenderableSpan {
                                                    span: rhs.error_span_wide(),
                                                    auxiliary: true,
                                                    note: Some(format!(
                                                        "There's a name binding `{}`.",
                                                        String::from_utf8_lossy(&unintern_string(rhs_name_binding, self.intermediate_dir).unwrap().unwrap_or(b"???".to_vec())),
                                                    )),
                                                },
                                            ],
                                            note: None,
                                        }]);
                                    },
                                    _ => todo!(),  // throw nice error message
                                };

                                lhs = Pattern {
                                    name: None,
                                    name_span: None,
                                    kind: PatternKind::InfixOp {
                                        op,
                                        lhs: Box::new(lhs),
                                        rhs: Box::new(rhs),
                                        op_span,
                                        kind: op_kind,
                                    },
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

    pub fn parse_patterns(&mut self, may_have_dot_dot: bool) -> Result<(Vec<Pattern>, Option<RestPattern>), Vec<Error>> {
        let mut rest = None;
        let mut patterns = vec![];

        loop {
            match self.peek4() {
                (
                    Some(Token { kind: TokenKind::Ident(_), .. }),
                    Some(Token { kind: TokenKind::Punct(Punct::At), .. }),
                    Some(Token { kind: TokenKind::Punct(Punct::DotDot), .. }),
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }) | None,
                ) | (
                    Some(Token { kind: TokenKind::Punct(Punct::DotDot), .. }),
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }) | None,
                    _,
                    _,
                ) => {
                    let (name, name_span, dot_dot_span) = match self.peek4() {
                        (
                            Some(Token { kind: TokenKind::Ident(name), span: name_span }),
                            Some(Token { kind: TokenKind::Punct(Punct::At), .. }),
                            Some(Token { kind: TokenKind::Punct(Punct::DotDot), span: dot_dot_span }),
                            Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }) | None,
                        ) => (Some(*name), Some(*name_span), *dot_dot_span),
                        (
                            Some(Token { kind: TokenKind::Punct(Punct::DotDot), span }),
                            Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }) | None,
                            _,
                            _,
                        ) => (None, None, *span),
                        _ => unreachable!(),
                    };

                    if !may_have_dot_dot {
                        return Err(vec![Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: ErrorToken::Pattern,
                                got: ErrorToken::Punct(Punct::DotDot),
                            },
                            spans: dot_dot_span.simple_error(),
                            note: Some(String::from("You cannot use a rest pattern (`..`) here. If you intended a range without ends, that's still invalid. A range must have at least one end, otherwise, just use a wildcard pattern.")),
                        }]);
                    }

                    else if let Some(RestPattern { span: prev_dot_dot_span, .. }) = rest {
                        return Err(vec![Error {
                            kind: ErrorKind::MultipleRestPatterns,
                            spans: vec![
                                RenderableSpan {
                                    span: dot_dot_span,
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
                        rest = Some(RestPattern {
                            span: dot_dot_span,
                            index: patterns.len(),
                            name,
                            name_span,
                        });
                        self.cursor += 2;
                    }
                },
                (Some(_), _, _, _) => {
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
                (None, _, _, _) => {
                    break;
                },
            }
        }

        Ok((patterns, rest))
    }
}

fn infix_binding_power(op: InfixOp) -> Option<(u32, u32)> {
    match op {
        InfixOp::Mul | InfixOp::Div | InfixOp::Rem => Some((MUL, MUL + 1)),
        InfixOp::Add | InfixOp::Sub => Some((ADD, ADD + 1)),
        InfixOp::Shl | InfixOp::Shr => Some((SHIFT, SHIFT + 1)),
        _ => None,
    }
}

fn concat_binding_power() -> (u32, u32) {
    (CONCAT, CONCAT + 1)
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

fn find_name_binding_in_const(pattern: &PatternKind) -> Option<InternedString> {
    match pattern {
        PatternKind::Ident { id, .. } => Some(*id),
        PatternKind::InfixOp { lhs, rhs, .. } => match find_name_binding_in_const(&lhs.kind) {
            Some(id) => Some(id),
            None => find_name_binding_in_const(&rhs.kind),
        },
        _ => None,
    }
}
