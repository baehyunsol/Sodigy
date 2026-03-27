use crate::{
    Block,
    Dotfish,
    Field,
    FuncArg,
    If,
    Lambda,
    Match,
    Path,
    StructInitField,
    Tokens,
    Type,
    merge_field_spans,
};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{
    Constant,
    Delim,
    InfixOp,
    Keyword,
    PostfixOp,
    PrefixOp,
    Punct,
    Token,
    TokenKind,
    TokensOrString,
};

mod from_pattern;

#[derive(Clone, Debug)]
pub enum Expr {
    Path(Path),
    Constant(Constant),
    If(Box<If>),
    Match(Box<Match>),
    Block(Box<Block>),
    Call {
        func: Box<Expr>,
        args: Vec<FuncArg>,
        arg_group_span: Span,
    },
    FormattedString {
        raw: bool,
        elements: Vec<ExprOrString>,
        span: Span,
    },
    Tuple {
        elements: Vec<Expr>,
        group_span: Span,
    },
    List {
        elements: Vec<Expr>,
        group_span: Span,
    },
    StructInit {
        constructor: Path,
        fields: Vec<StructInitField>,
        group_span: Span,
    },
    Field {
        lhs: Box<Expr>,
        field: Field,
        dotfish: Option<Dotfish>,
    },
    FieldUpdate {
        fields: Vec<Field>,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Lambda(Lambda),
    PrefixOp {
        op: PrefixOp,
        op_span: Span,
        rhs: Box<Expr>,
    },
    InfixOp {
        op: InfixOp,
        op_span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    PostfixOp {
        op: PostfixOp,
        op_span: Span,
        lhs: Box<Expr>,
    },
    TypeConversion {
        keyword_span: Span,
        lhs: Box<Expr>,
        rhs: Box<Type>,
        has_question_mark: bool,
    },

    // `x |> $ + 1` will become
    // `Pipeline { values: [Id(x), Add(PipelineData, Num(1))], spans: [span_of_the_first_op] }`

    // `pipe_spans[i]` is the span of `|>` after `values[i]`.
    Pipeline {
        values: Vec<Expr>,
        pipe_spans: Vec<Span>,
    },
    PipelineData(Span),  // `$`
}

impl Expr {
    pub fn error_span_narrow(&self) -> Span {
        match self {
            Expr::Path(p) => p.error_span_narrow(),
            Expr::Constant(c) => c.span(),
            Expr::FormattedString { span, .. } |
            Expr::Tuple { group_span: span, .. } |
            Expr::List { group_span: span, .. } |
            Expr::PrefixOp { op_span: span, .. } |
            Expr::InfixOp { op_span: span, .. } |
            Expr::PostfixOp { op_span: span, .. } |
            Expr::TypeConversion { keyword_span: span, .. } |
            Expr::PipelineData(span) => span.clone(),
            Expr::If(r#if) => r#if.if_span.clone(),
            Expr::Match(r#match) => r#match.keyword_span.clone(),
            Expr::Block(block) => block.group_span.clone(),
            Expr::Call { func, .. } => func.error_span_narrow(),
            Expr::StructInit { constructor, .. } => constructor.error_span_narrow(),
            Expr::Field { field, .. } => merge_field_spans(&[field.clone()]),
            Expr::FieldUpdate { fields, .. } => merge_field_spans(fields),
            Expr::Lambda(Lambda { arrow_span, .. }) => arrow_span.clone(),
            Expr::Pipeline { pipe_spans, .. } => pipe_spans[0].clone(),
        }
    }

    pub fn error_span_wide(&self) -> Span {
        match self {
            Expr::Path(p) => p.error_span_wide(),
            Expr::Constant(c) => c.span(),
            Expr::FormattedString { span, .. } |
            Expr::Tuple { group_span: span, .. } |
            Expr::List { group_span: span, .. } |
            Expr::PipelineData(span) => span.clone(),
            Expr::If(r#if) => r#if.if_span.merge(&r#if.true_group_span).merge(&r#if.false_group_span),
            Expr::Match(r#match) => r#match.keyword_span.merge(&r#match.group_span),
            Expr::Block(block) => block.group_span.clone(),
            Expr::Call { func, arg_group_span, .. } => func.error_span_wide().merge(arg_group_span),
            Expr::StructInit { constructor, group_span, .. } => constructor.error_span_wide().merge(group_span),

            // TODO: render dotfish operator
            Expr::Field { lhs, field, .. } => lhs.error_span_wide().merge(&merge_field_spans(&[field.clone()])),
            Expr::FieldUpdate { lhs, fields, rhs } => lhs.error_span_wide()
                .merge(&merge_field_spans(fields))
                .merge(&rhs.error_span_wide()),
            Expr::Lambda(Lambda { param_group_span, arrow_span, value, .. }) => param_group_span
                .merge(arrow_span)
                .merge(&value.error_span_wide()),
            Expr::PrefixOp { op_span, rhs, .. } => op_span.merge(&rhs.error_span_wide()),
            Expr::InfixOp { lhs, op_span, rhs, .. } => lhs.error_span_wide()
                .merge(op_span)
                .merge(&rhs.error_span_wide()),
            Expr::PostfixOp { lhs, op_span, .. } => lhs.error_span_wide().merge(op_span),
            Expr::TypeConversion { lhs, keyword_span, rhs, .. } => lhs.error_span_wide()
                .merge(keyword_span)
                .merge(&rhs.error_span_wide()),
            Expr::Pipeline { values, .. } => {
                let mut span = values[0].error_span_wide();

                for value in values.iter().skip(1) {
                    span = span.merge(&value.error_span_wide());
                }

                span
            },
        }
    }

    pub fn block_or_expr(block: Block) -> Expr {
        if block.lets.is_empty() && block.funcs.is_empty() &&
            block.structs.is_empty() && block.enums.is_empty() &&
            block.modules.is_empty() && block.uses.is_empty() &&
            block.value.is_some()
        {
            block.value.unwrap()
        }

        else {
            Expr::Block(Box::new(block))
        }
    }
}

impl<'t, 's> Tokens<'t, 's> {
    pub fn parse_expr(
        &mut self,
        try_struct_init: bool,
    ) -> Result<Expr, Vec<Error>> {
        self.pratt_parse(0, try_struct_init)
    }

    fn pratt_parse(
        &mut self,
        min_bp: u32,
        try_struct_init: bool,
    ) -> Result<Expr, Vec<Error>> {
        let mut lhs = match self.peek() {
            Some(Token { kind: TokenKind::Punct(Punct::Dollar), span }) => {
                let span = span.clone();
                self.cursor += 1;
                Expr::PipelineData(span)
            },
            Some(Token { kind: TokenKind::Punct(p), span }) => {
                let punct = *p;
                let punct_span = span.clone();

                match PrefixOp::try_from(punct) {
                    Ok(op) => {
                        let bp = prefix_binding_power(op);
                        self.cursor += 1;
                        let rhs = self.pratt_parse(bp, try_struct_init)?;
                        Expr::PrefixOp {
                            op,
                            op_span: punct_span,
                            rhs: Box::new(rhs),
                        }
                    },
                    Err(_) => {
                        return Err(vec![Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: ErrorToken::Expr,
                                got: ErrorToken::Punct(punct),
                            },
                            spans: punct_span.simple_error(),
                            note: Some(format!("`{}` is not a prefix operator.", p.render_error())),
                        }]);
                    },
                }
            },
            Some(Token { kind: TokenKind::Ident(id), span }) => {
                let (id, id_span) = (*id, span.clone());
                self.cursor += 1;
                Expr::Path(Path { id, id_span, fields: vec![], dotfish: vec![None] })
            },
            Some(Token { kind: TokenKind::Wildcard, span }) => {
                return Err(vec![Error {
                    kind: ErrorKind::WildcardNotAllowed,
                    spans: span.simple_error(),
                    note: None,
                }]);
            },
            Some(Token { kind: TokenKind::Number(n), span }) => {
                let (n, span) = (*n, span.clone());
                self.cursor += 1;
                Expr::Constant(Constant::Number { n, span })
            },
            Some(Token { kind: TokenKind::String { binary, regex: false, s, .. }, span }) => {
                let (binary, s, span) = (*binary, *s, span.clone());
                self.cursor += 1;
                Expr::Constant(Constant::String { binary, s, span })
            },
            Some(Token { kind: TokenKind::String { regex: true, s, .. }, span }) => todo!(),
            Some(Token { kind: TokenKind::Char(ch), span }) => {
                let (ch, span) = (*ch, span.clone());
                self.cursor += 1;
                Expr::Constant(Constant::Char { ch, span })
            },
            Some(Token { kind: TokenKind::Byte(b), span }) => {
                let (b, span) = (*b, span.clone());
                self.cursor += 1;
                Expr::Constant(Constant::Byte { b, span })
            },
            Some(Token { kind: TokenKind::FormattedString { raw, elements: token_elements }, span }) => {
                let (raw, span) = (*raw, span.clone());
                let mut elements = Vec::with_capacity(token_elements.len());

                for element in token_elements.iter() {
                    match element {
                        TokensOrString::String { s, span } => {
                            elements.push(ExprOrString::String { s: *s, span: span.clone() });
                        },
                        TokensOrString::Tokens { tokens, span } => {
                            let mut tokens = Tokens::new(tokens, span.end(), false, &self.intermediate_dir);
                            let expr = tokens.parse_expr(true)?;

                            // TODO: make sure that there's no remaining tokens
                            elements.push(ExprOrString::Expr(expr));
                        },
                    }
                }

                self.cursor += 1;
                Expr::FormattedString {
                    raw,
                    elements,
                    span,
                }
            },
            Some(Token { kind: TokenKind::Keyword(Keyword::If), .. }) => Expr::If(Box::new(self.parse_if_expr()?)),
            Some(Token { kind: TokenKind::Keyword(Keyword::Match), .. }) => Expr::Match(Box::new(self.parse_match_expr()?)),
            Some(Token { kind: TokenKind::Keyword(Keyword::Impure), span }) => {
                let impure_keyword_span = span.clone();
                self.cursor += 1;
                let mut lambda = self.parse_lambda()?;
                lambda.is_pure = false;
                lambda.impure_keyword_span = Some(impure_keyword_span);
                Expr::Lambda(lambda)
            },
            Some(Token { kind: TokenKind::Group { delim, tokens }, span }) => match delim {
                Delim::Lambda => Expr::Lambda(self.parse_lambda()?),
                Delim::Parenthesis => {
                    let span = span.clone();
                    let mut tokens = Tokens::new(tokens, span.end(), false, &self.intermediate_dir);
                    let exprs = tokens.parse_exprs()?;
                    let mut is_tuple = exprs.len() != 1;

                    // `(a + b)` is just an expression, but `(a + b,)` is a tuple
                    if exprs.len() == 1 && matches!(
                        tokens.last(),
                        Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }),
                    ) {
                        is_tuple = true;
                    }

                    self.cursor += 1;

                    if is_tuple {
                        Expr::Tuple {
                            elements: exprs,
                            group_span: span,
                        }
                    }

                    else {
                        exprs[0].clone()
                    }
                },
                Delim::Brace => {
                    let span = span.clone();
                    let mut tokens = Tokens::new(tokens, span.end(), false, &self.intermediate_dir);
                    let block = tokens.parse_block(false /* top_level */, span)?;
                    self.cursor += 1;

                    Expr::Block(Box::new(block))
                },
                Delim::Bracket => {
                    let span = span.clone();
                    let mut tokens = Tokens::new(tokens, span.end(), false, &self.intermediate_dir);
                    let exprs = tokens.parse_exprs()?;
                    self.cursor += 1;

                    Expr::List {
                        elements: exprs,
                        group_span: span,
                    }
                },
                Delim::Decorator | Delim::ModuleDecorator => {
                    return Err(vec![Error {
                        kind: ErrorKind::DecoratorNotAllowed,
                        spans: span.simple_error(),
                        note: Some(String::from("You cannot decorate an expression.")),
                    }]);
                },
            },
            Some(t) => {
                return Err(vec![Error {
                    kind: ErrorKind::UnexpectedToken {
                        expected: ErrorToken::Expr,
                        got: (&t.kind).into(),
                    },
                    spans: t.span.simple_error(),
                    note: None,
                }]);
            },
            None => {
                return Err(vec![self.unexpected_end(ErrorToken::Expr)]);
            },
        };

        loop {
            match self.peek() {
                Some(Token { kind: TokenKind::Punct(p), span }) => {
                    let punct = *p;
                    let punct_span = span.clone();

                    match PostfixOp::try_from(punct) {
                        // `..` and `..=` can be both infix and postfix!
                        Ok(op @ PostfixOp::Range { inclusive }) => {
                            let bp = postfix_binding_power(op);

                            if bp < min_bp {
                                break;
                            }

                            self.cursor += 1;

                            match self.peek().map(|t| t.expr_begin()) {
                                Some(true) => {
                                    let rhs = self.pratt_parse(bp, try_struct_init)?;
                                    lhs = Expr::InfixOp {
                                        op: InfixOp::Range { inclusive },
                                        op_span: punct_span,
                                        lhs: Box::new(lhs),
                                        rhs: Box::new(rhs),
                                    };
                                },
                                _ => {
                                    if inclusive {
                                        return Err(vec![Error {
                                            kind: ErrorKind::InclusiveRangeWithNoEnd,
                                            spans: punct_span.simple_error(),
                                            note: None,
                                        }]);
                                    }

                                    lhs = Expr::PostfixOp {
                                        op,
                                        op_span: punct_span,
                                        lhs: Box::new(lhs),
                                    };
                                },
                            }

                            continue;
                        },
                        Ok(op) => {
                            let bp = postfix_binding_power(op);

                            if bp < min_bp {
                                break;
                            }

                            self.cursor += 1;
                            lhs = Expr::PostfixOp {
                                op,
                                op_span: punct_span,
                                lhs: Box::new(lhs),
                            };
                            continue;
                        },
                        Err(_) => {
                            // let's try infix
                        },
                    }

                    // path
                    if let Punct::Dot = punct {
                        let (l_bp, _) = path_binding_power();

                        if l_bp < min_bp {
                            break;
                        }

                        self.cursor += 1;

                        if let Some(Token { kind: TokenKind::Punct(Punct::Lt), span }) = self.peek() {
                            let group_span_start = span.clone();
                            let (types, group_span_end) = self.parse_types_in_angle_brackets()?;
                            let group_span = group_span_start.merge(&group_span_end);

                            // `Option.<Int>`
                            if let Expr::Path(p) = &mut lhs {
                                *p.dotfish.last_mut().unwrap() = Some(Dotfish { types, group_span });
                            }

                            // `foo().bar.<Int>()`
                            else if let Expr::Field { dotfish, .. } = &mut lhs {
                                if dotfish.is_none() {
                                    *dotfish = Some(Dotfish { types, group_span });
                                }

                                // `foo().bar.<Int>.<Int>()`
                                else {
                                    todo!()
                                }
                            }

                            // `(x + y).<Int>`
                            else {
                                todo!()
                            }
                        }

                        else {
                            let (name, name_span) = self.pop_name_and_span(false /* allow_wildcard */)?;
                            let field = Field::Name {
                                name,
                                name_span,
                                dot_span: punct_span,
                                is_from_alias: false,
                            };

                            if let Expr::Path(p) = &mut lhs {
                                p.fields.push(field);
                                p.dotfish.push(None);
                            }

                            else {
                                lhs = Expr::Field {
                                    lhs: Box::new(lhs),
                                    field,
                                    dotfish: None,
                                };
                            }
                        }

                        continue;
                    }

                    match InfixOp::try_from(punct) {
                        Ok(op) => {
                            let (l_bp, r_bp) = infix_binding_power(op);

                            if l_bp < min_bp {
                                break;
                            }

                            self.cursor += 1;
                            let rhs = self.pratt_parse(r_bp, try_struct_init)?;

                            if op == InfixOp::Pipeline {
                                lhs = match (lhs, rhs) {
                                    (
                                        Expr::Pipeline { values: lhs_values, pipe_spans: lhs_pipe_spans },
                                        Expr::Pipeline { values: rhs_values, pipe_spans: rhs_pipe_spans },
                                    ) => {
                                        let new_values = vec![lhs_values, rhs_values].concat();
                                        let new_pipe_spans = vec![
                                            lhs_pipe_spans,
                                            vec![punct_span],
                                            rhs_pipe_spans,
                                        ].concat();

                                        Expr::Pipeline {
                                            values: new_values,
                                            pipe_spans: new_pipe_spans,
                                        }
                                    },
                                    (Expr::Pipeline { mut values, mut pipe_spans }, rhs) => {
                                        values.push(rhs);
                                        pipe_spans.push(punct_span);
                                        Expr::Pipeline { values, pipe_spans }
                                    },
                                    (lhs, Expr::Pipeline { values, pipe_spans }) => {
                                        let mut new_values = vec![lhs];
                                        let mut new_pipe_spans = vec![punct_span];
                                        new_values.extend(values);
                                        new_pipe_spans.extend(pipe_spans);

                                        Expr::Pipeline {
                                            values: new_values,
                                            pipe_spans: new_pipe_spans,
                                        }
                                    },
                                    (lhs, rhs) => Expr::Pipeline {
                                        values: vec![lhs, rhs],
                                        pipe_spans: vec![punct_span],
                                    },
                                };
                            }

                            else {
                                lhs = Expr::InfixOp {
                                    op,
                                    op_span: punct_span,
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                };
                            }

                            continue;
                        },
                        Err(_) => {
                            // Okay, `punct` is not an operator. we should not touch this.
                            break;
                        },
                    }
                },
                Some(Token { kind: TokenKind::Group { delim, tokens }, span }) => {
                    let span = span.clone();

                    match delim {
                        Delim::Lambda => {
                            return Err(vec![Error {
                                kind: ErrorKind::UnexpectedToken {
                                    expected: ErrorToken::Operator,
                                    got: ErrorToken::Group(*delim),
                                },
                                spans: span.simple_error(),
                                note: None,
                            }]);
                        },
                        Delim::Decorator | Delim::ModuleDecorator => {
                            return Err(vec![Error {
                                kind: ErrorKind::DecoratorNotAllowed,
                                spans: span.simple_error(),
                                note: Some(String::from("You cannot decorate an expression.")),
                            }]);
                        },
                        // call
                        Delim::Parenthesis => {
                            let (l_bp, _) = call_binding_power();

                            if l_bp < min_bp {
                                break;
                            }

                            let mut tokens = Tokens::new(tokens, span.end(), false, &self.intermediate_dir);
                            let args = tokens.parse_func_args()?;
                            self.cursor += 1;
                            lhs = Expr::Call {
                                func: Box::new(lhs),
                                args,
                                arg_group_span: span,
                            };
                            continue;
                        },
                        // struct initialization
                        // there are multiple cases:
                        // `if foo { 3 }` is valid, but it's not a struct initialization
                        // `if foo { bar: 3 }` is valid, and is a struct initialization
                        Delim::Brace => {
                            let (l_bp, _) = struct_init_binding_power();

                            if l_bp < min_bp || !try_struct_init {
                                break;
                            }

                            match &lhs {
                                Expr::Path(p) => {
                                    let constructor = p.clone();
                                    let mut tokens = Tokens::new(tokens, span.end(), false, &self.intermediate_dir);
                                    let fields = tokens.parse_struct_initialization()?;
                                    self.cursor += 1;
                                    lhs = Expr::StructInit {
                                        constructor,
                                        fields,
                                        group_span: span,
                                    };
                                    continue;
                                },
                                _ => {
                                    break;
                                },
                            }
                        },
                        // index
                        Delim::Bracket => {
                            let (l_bp, _) = infix_binding_power(InfixOp::Index);

                            if l_bp < min_bp {
                                break;
                            }

                            let mut tokens = Tokens::new(tokens, span.end(), false, &self.intermediate_dir);
                            let rhs = tokens.parse_expr(true)?;

                            // TODO: make sure that there's no remaining tokens

                            self.cursor += 1;
                            lhs = Expr::InfixOp {
                                op: InfixOp::Index,
                                op_span: span,
                                lhs: Box::new(lhs),
                                rhs: Box::new(rhs),
                            };
                            continue;
                        },
                    }
                },
                Some(Token {
                    kind: TokenKind::FieldUpdate { field, backtick_span, field_span },
                    ..
                }) => {
                    let (l_bp, r_bp) = field_update_binding_power();

                    if l_bp < min_bp {
                        break;
                    }

                    let mut fields = vec![Field::Name {
                        name: *field,
                        name_span: field_span.clone(),
                        dot_span: backtick_span.clone(),
                        is_from_alias: false,
                    }];
                    self.cursor += 1;

                    while let (
                        Some(Token { kind: TokenKind::Punct(Punct::Dot), span: dot_span }),
                        Some(Token { kind: TokenKind::Ident(id), span: field_span }),
                    ) = self.peek2() {
                        fields.push(Field::Name {
                            name: *id,
                            name_span: field_span.clone(),
                            dot_span: dot_span.clone(),
                            is_from_alias: false,
                        });
                        self.cursor += 2;
                    }

                    let rhs = self.pratt_parse(r_bp, try_struct_init)?;
                    lhs = Expr::FieldUpdate {
                        fields,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    };
                    continue;
                },
                Some(Token { kind: TokenKind::Keyword(Keyword::As), span }) => {
                    let (l_bp, r_bp) = as_binding_power();
                    let mut keyword_span = span.clone();

                    if l_bp < min_bp {
                        break;
                    }

                    self.cursor += 1;

                    let has_question_mark = match self.peek() {
                        Some(Token { kind: TokenKind::Punct(Punct::QuestionMark), span }) => {
                            keyword_span = keyword_span.merge(span);
                            self.cursor += 1;
                            true
                        },
                        _ => false,
                    };
                    let (types, _) = self.parse_types_in_angle_brackets()?;

                    if types.len() != 1 {
                        // `x as <Int, Int>` is an error
                        //
                        // `parse_types_in_angle_brackets` guarantees that `types` is not empty
                        todo!();
                    }

                    lhs = Expr::TypeConversion {
                        keyword_span,
                        lhs: Box::new(lhs),
                        rhs: Box::new(types[0].clone()),
                        has_question_mark,
                    };
                    continue;
                },
                _ => {
                    break;
                },
            }
        }

        Ok(lhs)
    }

    pub fn parse_exprs(&mut self) -> Result<Vec<Expr>, Vec<Error>> {
        let mut exprs = vec![];

        if self.peek().is_none() {
            return Ok(exprs);
        }

        loop {
            exprs.push(self.parse_expr(true)?);

            match self.peek2() {
                (
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }),
                    Some(_),
                ) => {
                    self.cursor += 1;
                },
                (
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }),
                    None,
                ) => {
                    self.cursor += 1;
                    break;
                },
                (None, _) => {
                    break;
                },
                (Some(t), _) => {
                    return Err(vec![Error {
                        kind: ErrorKind::UnexpectedToken {
                            expected: ErrorToken::Punct(Punct::Comma),
                            got: (&t.kind).into(),
                        },
                        spans: t.span.simple_error(),
                        note: None,
                    }]);
                },
            }
        }

        Ok(exprs)
    }
}

#[derive(Clone, Debug)]
pub enum ExprOrString {
    Expr(Expr),
    String { s: InternedString, span: Span },
}

fn path_binding_power() -> (u32, u32) {
    (PATH, PATH + 1)
}

fn struct_init_binding_power() -> (u32, u32) {
    (STRUCT_INIT, STRUCT_INIT + 1)
}

fn call_binding_power() -> (u32, u32) {
    (CALL, CALL + 1)
}

fn as_binding_power() -> (u32, u32) {
    (AS, AS + 1)
}

fn field_update_binding_power() -> (u32, u32) {
    (UPDATE, UPDATE + 1)
}

fn prefix_binding_power(op: PrefixOp) -> u32 {
    match op {
        PrefixOp::Not | PrefixOp::Neg => NEG,
        PrefixOp::Range { .. } => RANGE,
    }
}

fn infix_binding_power(op: InfixOp) -> (u32, u32) {
    match op {
        InfixOp::Index => (INDEX, INDEX + 1),
        InfixOp::Mul | InfixOp::Div | InfixOp::Rem => (MUL, MUL + 1),
        InfixOp::Add | InfixOp::Sub => (ADD, ADD + 1),
        InfixOp::Shl | InfixOp::Shr => (SHIFT, SHIFT + 1),
        InfixOp::BitAnd => (BIT_AND, BIT_AND + 1),
        InfixOp::Xor => (XOR, XOR + 1),
        InfixOp::BitOr => (BIT_OR, BIT_OR + 1),
        InfixOp::Range { .. } => (RANGE, RANGE + 1),
        InfixOp::Append => (APPEND, APPEND + 1),
        InfixOp::Prepend => (PREPEND, PREPEND + 1),
        InfixOp::Concat => (CONCAT, CONCAT + 1),
        InfixOp::Lt | InfixOp::Gt | InfixOp::Leq | InfixOp::Geq => (COMP, COMP + 1),
        InfixOp::Eq | InfixOp::Neq => (COMP_EQ, COMP_EQ + 1),
        InfixOp::LogicAnd => (LOGIC_AND, LOGIC_AND + 1),
        InfixOp::LogicOr => (LOGIC_OR, LOGIC_OR + 1),
        InfixOp::Pipeline => (PIPELINE, PIPELINE + 1),
    }
}

fn postfix_binding_power(op: PostfixOp) -> u32 {
    match op {
        PostfixOp::Range { .. } => RANGE,
        PostfixOp::QuestionMark => QUESTION,
    }
}

const PATH: u32 = 41;  // a.b
const STRUCT_INIT: u32 = 39;  // foo { a: 1, b: 2 }
const CALL: u32 = 37;  // foo()
const INDEX: u32 = 35;  // a[3]
const QUESTION: u32 = 33;  // a?
const NEG: u32 = 31;  // -a, !a
const AS: u32 = 29;
const MUL: u32 = 27;  // a * b, a / b, a % b
const ADD: u32 = 25;  // a + b, a - b
const SHIFT: u32 = 23;  // a << b, a >> b
const BIT_AND: u32 = 21;  // a & b
const XOR: u32 = 19;  // a ^ b
const BIT_OR: u32 = 17;  // a | b

// TODO: it has to be right associative...
const APPEND: u32 = 15; const PREPEND: u32 = 15;

// RANGE: a..b, a..=b, a.., ..a
const CONCAT: u32 = 13; const RANGE: u32 = 13;
const COMP: u32 = 11;  // a < b, a > b, a <= b, a >= b
const COMP_EQ: u32 = 9;  // a == b, a != b
const UPDATE: u32 = 7;  // p `age 32
const LOGIC_AND: u32 = 5;  // a && b
const LOGIC_OR: u32 = 3;  // a || b
const PIPELINE: u32 = 1;  // x |> $ + 1
