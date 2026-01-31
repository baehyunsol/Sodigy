use crate::{Field, Path, Tokens};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_number::InternedNumber;
use sodigy_span::{RenderableSpan, Span, SpanDeriveKind};
use sodigy_string::InternedString;
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
    // An identifier without fields is also a path.
    Path(Path),

    // `if let Some($x) = foo() { .. }`
    NameBinding {
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
    Struct {
        r#struct: Path,
        fields: Vec<StructFieldPattern>,
        rest: Option<RestPattern>,
        group_span: Span,
    },
    TupleStruct {
        r#struct: Path,
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
    // TODO: prefix/postfix op

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

    // Every operand is a constant or an identifier (value), like `Some(x + y + 1)`.
    Value,

    // Exactly one operand is a name binding and the other operands are constants, like `Some($x + (1 << 32))`.
    NameBinding,
}

#[derive(Clone, Debug)]
pub struct StructFieldPattern {
    pub name: InternedString,
    pub span: Span,
    pub pattern: Pattern,
    pub is_shorthand: bool,
}

// `..` in `[$a, $b, .., $c]`
// Parser guarantees that there's at most 1 rest in a group.
#[derive(Clone, Copy, Debug)]
pub struct RestPattern {
    pub span: Span,
    pub index: usize,

    // You can bind a name to dot_dot.
    // `[$n] ++ $ns` is lowered to `[$n, $ns @ ..]`.
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

    // It's used to lower `[$n] ++ $ns` to `[$n, $ns @ ..]`.
    // Lhs and rhs of the concat operator are converted to a list pattern,
    // then their elements are concatonated.
    // - `$ns` -> `[$ns @ ..]`
    // - `[$a, $b, $c]` -> `[$a, $b, $c]`
    // - `"asdf"` -> `['a', 'b', 'c', 'd']`
    // - `[$a] ++ [$b]` -> `[$a, $b]`
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
            PatternKind::NameBinding { id, span } => PatternKind::List {
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
            PatternKind::NameBinding { span, .. } |
            PatternKind::Number { span, .. } |
            PatternKind::String { span, .. } |
            PatternKind::Regex { span, .. } |
            PatternKind::Char { span, .. } |
            PatternKind::Byte { span, .. } |
            PatternKind::Wildcard(span) |
            PatternKind::PipelineData(span) |
            PatternKind::Tuple { group_span: span, .. } |
            PatternKind::List { group_span: span, .. } |
            PatternKind::Range { op_span: span, .. } |
            PatternKind::Or { op_span: span, .. } |
            PatternKind::InfixOp { op_span: span, .. } => *span,
            PatternKind::Path(path) |
            PatternKind::Struct { r#struct: path, .. } |
            PatternKind::TupleStruct { r#struct: path, .. } => path.error_span_narrow(),
        }
    }

    pub fn error_span_wide(&self) -> Span {
        match self {
            PatternKind::Path(p) => p.error_span_wide(),
            PatternKind::NameBinding { span, .. } |
            PatternKind::Number { span, .. } |
            PatternKind::String { span, .. } |
            PatternKind::Regex { span, .. } |
            PatternKind::Char { span, .. } |
            PatternKind::Byte { span, .. } |
            PatternKind::Wildcard(span) |
            PatternKind::PipelineData(span) |
            PatternKind::Tuple { group_span: span, .. } |
            PatternKind::List { group_span: span, .. } => *span,
            PatternKind::Struct { r#struct, group_span, .. } |
            PatternKind::TupleStruct { r#struct, group_span, .. } => r#struct.error_span_wide().merge(*group_span),
            PatternKind::Range { lhs, op_span, rhs, .. } => {
                let mut span = match lhs {
                    Some(lhs) => lhs.error_span_wide().merge(*op_span),
                    None => *op_span,
                };

                if let Some(rhs) = rhs {
                    span = span.merge(rhs.error_span_wide());
                }

                span
            },
            PatternKind::InfixOp { lhs, op_span, rhs, .. } |
            PatternKind::Or { lhs, op_span, rhs } => lhs.error_span_wide()
                .merge(*op_span)
                .merge(rhs.error_span_wide()),
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
            PatternKind::PipelineData(_) => vec![],
            PatternKind::NameBinding { id, span } => vec![(*id, *span)],
            PatternKind::Struct { fields, rest, .. } => {
                let mut result = fields.iter().flat_map(|f| f.pattern.bound_names()).collect::<Vec<_>>();

                if let Some(rest) = rest {
                    if let (Some(name), Some(name_span)) = (rest.name, rest.name_span) {
                        result.push((name, name_span));
                    }
                }

                result
            },
            PatternKind::TupleStruct { elements, rest, .. } |
            PatternKind::Tuple { elements, rest, .. } |
            PatternKind::List { elements, rest, .. } => {
                let mut result = elements.iter().flat_map(|e| e.bound_names()).collect::<Vec<_>>();

                if let Some(rest) = rest {
                    if let (Some(name), Some(name_span)) = (rest.name, rest.name_span) {
                        result.push((name, name_span));
                    }
                }

                result
            },
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

impl ParsePatternContext {
    pub fn expected_token(&self) -> ErrorToken {
        match self {
            ParsePatternContext::MatchArm => ErrorToken::Punct(Punct::Arrow),
            ParsePatternContext::IfLet | ParsePatternContext::Let => ErrorToken::Punct(Punct::Assign),
            ParsePatternContext::Group => ErrorToken::Punct(Punct::Comma),
        }
    }
}

impl<'t, 's> Tokens<'t, 's> {
    pub fn parse_pattern(&mut self, context: ParsePatternContext) -> Result<Pattern, Vec<Error>> {
        self.pratt_parse_pattern(context, 0)
    }

    // It only catches errors that make it im-parse-able.
    // All the other checks are done by `Pattern::check()`.
    fn pratt_parse_pattern(&mut self, context: ParsePatternContext, min_bp: u32) -> Result<Pattern, Vec<Error>> {
        let mut lhs = match self.peek2() {
            (Some(Token { kind: TokenKind::Ident(id), span }), _) => {
                let (id, id_span) = (*id, *span);
                self.cursor += 1;

                if id.eq(b"_") {
                    Pattern {
                        name: None,
                        name_span: None,
                        kind: PatternKind::Wildcard(id_span),
                    }
                }

                else {
                    // no dotfish operators in patterns
                    let mut fields = vec![];
                    let mut types = vec![None];

                    loop {
                        match self.peek2() {
                            (
                                Some(Token { kind: TokenKind::Punct(Punct::Dot), span: dot_span }),
                                Some(Token { kind: TokenKind::Ident(id), span }),
                            ) => {
                                fields.push(Field::Name {
                                    name: *id,
                                    name_span: *span,
                                    dot_span: *dot_span,
                                    is_from_alias: false,
                                });
                                types.push(None);
                                self.cursor += 2;
                            },
                            _ => break,
                        }
                    }

                    Pattern {
                        name: None,
                        name_span: None,
                        kind: PatternKind::Path(Path { id, id_span, fields, types }),
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
                    kind: PatternKind::NameBinding { id, span },
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
            (Some(t), _) => {
                return Err(vec![Error {
                    kind: ErrorKind::UnexpectedToken {
                        expected: ErrorToken::Pattern,
                        got: (&t.kind).into(),
                    },
                    spans: t.span.simple_error(),
                    note: None,
                }]);
            },
            (None, _) => {
                return Err(vec![self.unexpected_end(ErrorToken::Pattern)]);
            },
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
                                    kind: PatternKind::NameBinding { id, span },
                                } => {
                                    name_binding = Some((id, span));

                                    // `$a @ $b @ 1..2`
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
                                    return Err(vec![Error {
                                        kind: ErrorKind::UnexpectedToken {
                                            expected: context.expected_token(),
                                            got: ErrorToken::Punct(Punct::At),
                                        },
                                        spans: op_span.simple_error(),
                                        note: None,
                                    }]);
                                },
                            }

                            match &rhs {
                                // `$a @ $b @ 1..2`
                                Pattern { name: Some(name), name_span: Some(name_span), .. } |
                                // `$a @ $b`
                                Pattern { kind: PatternKind::NameBinding { id: name, span: name_span }, .. } => {
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
                                Pattern { kind: PatternKind::Path(path), .. } => {
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
                                                span: path.error_span_wide(),
                                                auxiliary: false,
                                                note: Some(format!(
                                                    "It already has a name `{}`. You can just use this name.",
                                                    path.unintern_or_default(&self.intermediate_dir),
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
                                        PatternKind::Path { .. } | PatternKind::Number { .. } | PatternKind::Char { .. } | PatternKind::Byte { .. } | PatternKind::InfixOp { kind: PatternValueKind::Constant | PatternValueKind::Value, .. },
                                        PatternKind::Path { .. } | PatternKind::Number { .. } | PatternKind::Char { .. } | PatternKind::Byte { .. } | PatternKind::InfixOp { kind: PatternValueKind::Constant | PatternValueKind::Value, .. },
                                    ) => PatternValueKind::Value,
                                    (
                                        PatternKind::NameBinding { .. } | PatternKind::InfixOp { kind: PatternValueKind::NameBinding, .. },
                                        PatternKind::Number { .. } | PatternKind::Char { .. } | PatternKind::Byte { .. } | PatternKind::InfixOp { kind: PatternValueKind::Constant, .. },
                                    ) | (
                                        PatternKind::Number { .. } | PatternKind::Char { .. } | PatternKind::Byte { .. } | PatternKind::InfixOp { kind: PatternValueKind::Constant, .. },
                                        PatternKind::NameBinding { .. } | PatternKind::InfixOp { kind: PatternValueKind::NameBinding, .. },
                                    ) => {
                                        let (aux_span, aux_name) = match (&lhs.kind, &rhs.kind) {
                                            (PatternKind::NameBinding { .. } | PatternKind::InfixOp { kind: PatternValueKind::NameBinding, .. }, _) => {
                                                (lhs.error_span_wide(), find_name_binding_in_const(&lhs.kind).unwrap())
                                            },
                                            (_, PatternKind::NameBinding { .. } | PatternKind::InfixOp { kind: PatternValueKind::NameBinding, .. }) => {
                                                (rhs.error_span_wide(), find_name_binding_in_const(&rhs.kind).unwrap())
                                            },
                                            _ => unreachable!(),
                                        };

                                        // `let $x + 1 = 100;` is okay.
                                        // `let $x * 2 = 100;` is not.
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
                                                                aux_name.unintern_or_default(&self.intermediate_dir),
                                                            )),
                                                        },
                                                    ],
                                                    note: None,
                                                }]);
                                            },
                                        }

                                        PatternValueKind::NameBinding
                                    },
                                    (
                                        PatternKind::NameBinding { .. } | PatternKind::InfixOp { kind: PatternValueKind::NameBinding, .. },
                                        PatternKind::NameBinding { .. } | PatternKind::InfixOp { kind: PatternValueKind::NameBinding, .. },
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
                                                        lhs_name_binding.unintern_or_default(&self.intermediate_dir),
                                                    )),
                                                },
                                                RenderableSpan {
                                                    span: rhs.error_span_wide(),
                                                    auxiliary: true,
                                                    note: Some(format!(
                                                        "There's a name binding `{}`.",
                                                        rhs_name_binding.unintern_or_default(&self.intermediate_dir),
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
                                let (expected, note) = match p {
                                    Punct::Eq => match context {
                                        ParsePatternContext::Let | ParsePatternContext::IfLet => (
                                            ErrorToken::Punct(Punct::Assign),
                                            Some("Use `=` instead of `==` here."),
                                        ),
                                        ParsePatternContext::MatchArm => (
                                            ErrorToken::Punct(Punct::Arrow),
                                            Some("Use `=>` instead of `==` here."),
                                        ),
                                        ParsePatternContext::Group => (
                                            ErrorToken::Punct(Punct::Comma),
                                            Some("You can't assign anything here"),
                                        ),
                                    },
                                    Punct::ReturnType => match context {
                                        ParsePatternContext::Let | ParsePatternContext::IfLet => (
                                            ErrorToken::Punct(Punct::Assign),
                                            None,
                                        ),
                                        ParsePatternContext::MatchArm => (
                                            ErrorToken::Punct(Punct::Arrow),
                                            Some("Use `=>` instead of `->` here."),
                                        ),
                                        ParsePatternContext::Group => (
                                            ErrorToken::Punct(Punct::Comma),
                                            None,
                                        ),
                                    },
                                    _ if context.expected_token().unwrap_punct() == p => {
                                        break;
                                    },
                                    _ => match context {
                                        ParsePatternContext::Let | ParsePatternContext::IfLet => (
                                            ErrorToken::Punct(Punct::Assign),
                                            None,
                                        ),
                                        ParsePatternContext::MatchArm => (
                                            ErrorToken::Punct(Punct::Arrow),
                                            None,
                                        ),
                                        ParsePatternContext::Group => (
                                            ErrorToken::Punct(Punct::Comma),
                                            None,
                                        ),
                                    },
                                };

                                return Err(vec![Error {
                                    kind: ErrorKind::UnexpectedToken {
                                        expected,
                                        got: ErrorToken::Punct(p),
                                    },
                                    spans: vec![
                                        RenderableSpan {
                                            span: op_span,
                                            auxiliary: false,
                                            note: note.map(|n| n.to_string()),
                                        },
                                    ],
                                    note: None,
                                }]);
                            },
                        },
                    }
                },
                // struct or a tuple_struct
                Some(t @ Token { kind: TokenKind::Group { delim, tokens }, span }) => {
                    let r#struct = match &lhs.kind {
                        PatternKind::Path(path) => path.clone(),
                        _ => {
                            return Err(vec![Error {
                                kind: ErrorKind::UnexpectedToken {
                                    expected: context.expected_token(),
                                    got: (&t.kind).into(),
                                },
                                spans: span.simple_error(),
                                note: None,
                            }]);
                        },
                    };

                    match delim {
                        Delim::Parenthesis => {
                            let group_span = *span;
                            let mut tokens = Tokens::new(tokens, span.end(), &self.intermediate_dir);
                            let (elements, rest) = tokens.parse_patterns(/* may_have_dot_dot: */ true)?;
                            self.cursor += 1;

                            lhs = Pattern {
                                name: None,
                                name_span: None,
                                kind: PatternKind::TupleStruct {
                                    r#struct,
                                    elements,
                                    group_span,
                                    rest,
                                },
                            };
                        },
                        Delim::Brace => todo!(),
                        _ => {
                            return Err(vec![Error {
                                kind: ErrorKind::UnexpectedToken {
                                    expected: ErrorToken::BraceOrParenthesis,
                                    got: (&t.kind).into(),
                                },
                                spans: span.simple_error(),
                                note: None,
                            }]);
                        },
                    }
                },
                Some(t) => {
                    return Err(vec![Error {
                        kind: ErrorKind::UnexpectedToken {
                            expected: context.expected_token(),
                            got: (&t.kind).into(),
                        },
                        spans: t.span.simple_error(),
                        note: None,
                    }]);
                },
                None => match context {
                    ParsePatternContext::Group => {
                        break;
                    },
                    _ => {
                        return Err(vec![self.unexpected_end(context.expected_token())]);
                    },
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
        PatternKind::NameBinding { id, .. } => Some(*id),
        PatternKind::InfixOp { lhs, rhs, .. } => match find_name_binding_in_const(&lhs.kind) {
            Some(id) => Some(id),
            None => find_name_binding_in_const(&rhs.kind),
        },
        _ => None,
    }
}
