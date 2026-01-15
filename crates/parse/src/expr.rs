use crate::{
    Block,
    FuncArg,
    If,
    Lambda,
    Match,
    StructInitField,
    Tokens,
};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{
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

        // it includes quotes
        span: Span,
    },
    Char {
        ch: u32,

        // it includes quotes
        span: Span,
    },
    Byte {
        b: u8,
        span: Span,
    },
    If(If),
    Match(Match),
    Block(Block),
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
        r#struct: Box<Expr>,
        fields: Vec<StructInitField>,
        group_span: Span,
    },
    Path {
        lhs: Box<Expr>,
        field: Field,
    },
    FieldModifier {
        fields: Vec<(InternedString, Span)>,
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
            Expr::Ident { span, .. } |
            Expr::Number { span, .. } |
            Expr::String { span, .. } |
            Expr::Char { span, .. } |
            Expr::Byte { span, .. } |
            Expr::FormattedString { span, .. } |
            Expr::Tuple { group_span: span, .. } |
            Expr::List { group_span: span, .. } |
            Expr::PrefixOp { op_span: span, .. } |
            Expr::InfixOp { op_span: span, .. } |
            Expr::PostfixOp { op_span: span, .. } |
            Expr::PipelineData(span) => *span,
            Expr::If(r#if) => r#if.if_span,
            Expr::Match(r#match) => r#match.keyword_span,
            Expr::Block(block) => block.group_span,
            Expr::Call { func, .. } => func.error_span_narrow(),
            Expr::StructInit { r#struct, .. } => r#struct.error_span_narrow(),
            Expr::Path { field, .. } => {
                let Field::Name { dot_span, .. } = field else { unreachable!() };
                *dot_span
            },
            Expr::FieldModifier { fields, .. } => {
                let mut span = fields[0].1;

                for (_, field_span) in fields.iter().skip(1) {
                    span = span.merge(*field_span);
                }

                span
            },
            Expr::Lambda(Lambda { arrow_span, .. }) => *arrow_span,
            Expr::Pipeline { pipe_spans, .. } => pipe_spans[0],
        }
    }

    pub fn error_span_wide(&self) -> Span {
        match self {
            Expr::Ident { span, .. } |
            Expr::Number { span, .. } |
            Expr::String { span, .. } |
            Expr::Char { span, .. } |
            Expr::Byte { span, .. } |
            Expr::FormattedString { span, .. } |
            Expr::Tuple { group_span: span, .. } |
            Expr::List { group_span: span, .. } |
            Expr::PipelineData(span) => *span,
            Expr::If(r#if) => r#if.if_span.merge(r#if.true_group_span).merge(r#if.false_group_span),
            Expr::Match(r#match) => r#match.keyword_span.merge(r#match.group_span),
            Expr::Block(block) => block.group_span,
            Expr::Call { func, arg_group_span, .. } => func.error_span_wide().merge(*arg_group_span),
            Expr::StructInit { r#struct, group_span, .. } => r#struct.error_span_narrow().merge(*group_span),
            Expr::Path { lhs, field } => {
                let span = lhs.error_span_wide();

                match field {
                    Field::Name { span: s, .. } => span.merge(*s),
                    _ => unreachable!(),
                }
            },
            Expr::FieldModifier { lhs, fields, rhs } => {
                let mut span = lhs.error_span_wide();

                for (_, field_span) in fields.iter() {
                    span = span.merge(*field_span);
                }

                span.merge(rhs.error_span_wide())
            },
            Expr::Lambda(Lambda { param_group_span, arrow_span, value, .. }) => param_group_span
                .merge(*arrow_span)
                .merge(value.error_span_wide()),
            Expr::PrefixOp { op_span, rhs, .. } => op_span.merge(rhs.error_span_wide()),
            Expr::InfixOp { lhs, op_span, rhs, .. } => lhs.error_span_wide()
                .merge(*op_span)
                .merge(rhs.error_span_wide()),
            Expr::PostfixOp { lhs, op_span, .. } => lhs.error_span_wide().merge(*op_span),
            Expr::Pipeline { values, .. } => {
                let mut span = values[0].error_span_wide();

                for value in values.iter().skip(1) {
                    span = span.merge(value.error_span_wide());
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
            Expr::Block(block)
        }
    }
}

/// Variants other than `Name` are generated by the compiler.
/// Even though the user code is `a._0`, it won't be parsed to `Field::Index(0)`.
/// It'll first be parsed to `Field::Name("_0")`, then very later (after mir) lowered
/// to `Field::Index(0)`.
#[derive(Clone, Copy, Debug)]
pub enum Field {
    Name {
        name: InternedString,
        span: Span,
        dot_span: Span,
        is_from_alias: bool,
    },

    /// In `let (_, x) = foo()`, `x` is `Index(1)` of `foo()`.
    /// In `let (_, _, .., x) = foo()`, `x` is `Index(-1)` of `foo()`.
    Index(i64),

    /// 1. In `let (_, _, x @ .., _, _, _) = foo()`, `x` is `Range(2, -3)` of `foo()`.
    /// 2. In `let ([_] ++ x ++ [_]) = foo()`, `x` is `Range(1, -1)` of `foo()`.
    ///
    /// I'm not sure whether I should implement 1, but 2 must be implemented.
    Range(i64, i64),

    /// Returns a variant index of an enum value.
    Variant,

    /// Special field for pattern analysis.
    Constructor,

    /// Special field for pattern analysis.
    Payload,
}

impl Field {
    pub fn dot_span(&self) -> Option<Span> {
        match self {
            Field::Name { dot_span, .. } => Some(*dot_span),
            Field::Index(_) |
            Field::Range(_, _) |
            Field::Variant |
            Field::Constructor |
            Field::Payload => None,
        }
    }

    pub fn unwrap_name(&self) -> InternedString {
        match self {
            Field::Name { name, .. } => *name,
            _ => panic!(),
        }
    }

    pub fn unwrap_span(&self) -> Span {
        match self {
            Field::Name { span, .. } => *span,
            _ => panic!(),
        }
    }
}

impl<'t, 's> Tokens<'t, 's> {
    pub fn parse_expr(&mut self) -> Result<Expr, Vec<Error>> {
        self.pratt_parse(0)
    }

    fn pratt_parse(&mut self, min_bp: u32) -> Result<Expr, Vec<Error>> {
        let mut lhs = match self.peek() {
            Some(Token { kind: TokenKind::Punct(Punct::Dollar), span }) => {
                let span = *span;
                self.cursor += 1;
                Expr::PipelineData(span)
            },
            Some(Token { kind: TokenKind::Punct(p), span }) => {
                let punct = *p;
                let punct_span = *span;

                match PrefixOp::try_from(punct) {
                    Ok(op) => {
                        let bp = prefix_binding_power(op);
                        self.cursor += 1;
                        let rhs = self.pratt_parse(bp)?;
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
                let (id, span) = (*id, *span);
                self.cursor += 1;
                Expr::Ident { id, span }
            },
            Some(Token { kind: TokenKind::Number(n), span }) => {
                let (n, span) = (n.clone(), *span);
                self.cursor += 1;
                Expr::Number { n, span }
            },
            Some(Token { kind: TokenKind::String { binary, regex: false, s, .. }, span }) => {
                let (binary, s, span) = (*binary, *s, *span);
                self.cursor += 1;
                Expr::String { binary, s, span }
            },
            Some(Token { kind: TokenKind::String { regex: true, s, .. }, span }) => todo!(),
            Some(Token { kind: TokenKind::Char(ch), span }) => {
                let (ch, span) = (*ch, *span);
                self.cursor += 1;
                Expr::Char { ch, span }
            },
            Some(Token { kind: TokenKind::Byte(b), span }) => {
                let (b, span) = (*b, *span);
                self.cursor += 1;
                Expr::Byte { b, span }
            },
            Some(Token { kind: TokenKind::FormattedString { raw, elements: token_elements }, span }) => {
                let (raw, span) = (*raw, *span);
                let mut elements = Vec::with_capacity(token_elements.len());

                for element in token_elements.iter() {
                    match element {
                        TokensOrString::String { s, span } => {
                            elements.push(ExprOrString::String { s: *s, span: *span });
                        },
                        TokensOrString::Tokens { tokens, span } => {
                            let mut tokens = Tokens::new(tokens, span.end(), &self.intermediate_dir);
                            let expr = tokens.parse_expr()?;

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
            Some(Token { kind: TokenKind::Keyword(Keyword::If), .. }) => Expr::If(self.parse_if_expr()?),
            Some(Token { kind: TokenKind::Keyword(Keyword::Match), .. }) => Expr::Match(self.parse_match_expr()?),
            Some(Token { kind: TokenKind::Keyword(Keyword::Impure), span }) => {
                let impure_keyword_span = *span;
                self.cursor += 1;
                let mut lambda = self.parse_lambda()?;
                lambda.is_pure = false;
                lambda.impure_keyword_span = Some(impure_keyword_span);
                Expr::Lambda(lambda)
            },
            Some(Token { kind: TokenKind::Group { delim, tokens }, span }) => match delim {
                Delim::Lambda => Expr::Lambda(self.parse_lambda()?),
                Delim::Parenthesis => {
                    let span = *span;
                    let mut tokens = Tokens::new(tokens, span.end(), &self.intermediate_dir);
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
                    let span = *span;
                    let mut tokens = Tokens::new(tokens, span.end(), &self.intermediate_dir);
                    let block = tokens.parse_block(false /* top_level */, span)?;
                    self.cursor += 1;

                    Expr::Block(block)
                },
                Delim::Bracket => {
                    let span = *span;
                    let mut tokens = Tokens::new(tokens, span.end(), &self.intermediate_dir);
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
            Some(t) => panic!("TODO: {t:?}"),
            None => {
                return Err(vec![self.unexpected_end(ErrorToken::Expr)]);
            },
        };

        loop {
            match self.peek() {
                Some(Token {
                    kind: TokenKind::Punct(p),
                    span,
                }) => {
                    let punct = *p;
                    let punct_span = *span;

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
                                    let rhs = self.pratt_parse(bp)?;
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

                    // path operator
                    if let Punct::Dot = punct {
                        let (l_bp, _) = path_binding_power();

                        if l_bp < min_bp {
                            break;
                        }

                        self.cursor += 1;
                        let (name, name_span) = self.pop_name_and_span()?;
                        lhs = Expr::Path {
                            lhs: Box::new(lhs),
                            field: Field::Name {
                                name,
                                span: name_span,
                                dot_span: punct_span,
                                is_from_alias: false,
                            },
                        };
                        continue;
                    }

                    match InfixOp::try_from(punct) {
                        Ok(op) => {
                            let (l_bp, r_bp) = infix_binding_power(op);

                            if l_bp < min_bp {
                                break;
                            }

                            self.cursor += 1;
                            let rhs = self.pratt_parse(r_bp)?;

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
                Some(Token {
                    kind: TokenKind::Group { delim, tokens },
                    span,
                }) => {
                    let span = *span;

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

                            let mut tokens = Tokens::new(tokens, span.end(), &self.intermediate_dir);
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

                            if l_bp < min_bp {
                                break;
                            }

                            let mut tokens = Tokens::new(tokens, span.end(), &self.intermediate_dir);

                            match tokens.try_parse_struct_initialization() {
                                Some(Ok(s)) => {
                                    self.cursor += 1;
                                    lhs = Expr::StructInit {
                                        r#struct: Box::new(lhs),
                                        fields: s,
                                        group_span: span,
                                    };
                                    continue;
                                },

                                // it's a struct initialization,
                                // but there's a synax error
                                Some(Err(e)) => {
                                    return Err(e);
                                },

                                // not a struct initialization
                                None => {
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

                            let mut tokens = Tokens::new(tokens, span.end(), &self.intermediate_dir);
                            let rhs = tokens.parse_expr()?;

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
                    kind: TokenKind::FieldModifier(field),
                    span,
                }) => {
                    let (l_bp, r_bp) = field_modifier_binding_power();

                    if l_bp < min_bp {
                        break;
                    }

                    let mut fields = vec![(*field, *span)];
                    self.cursor += 1;

                    while let Some(Token {
                        kind: TokenKind::FieldModifier(field),
                        span,
                    }) = self.peek() {
                        fields.push((*field, *span));
                        self.cursor += 1;
                    }

                    let rhs = self.pratt_parse(r_bp)?;
                    lhs = Expr::FieldModifier {
                        fields,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    };
                    continue;
                },
                Some(t) => panic!("TODO: {t:?}"),
                None => {
                    return Ok(lhs);
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
            exprs.push(self.parse_expr()?);

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

fn field_modifier_binding_power() -> (u32, u32) {
    (MODIFY, MODIFY + 1)
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

const PATH: u32 = 39;  // a.b
const STRUCT_INIT: u32 = 37;  // foo { a: 1, b: 2 }
const CALL: u32 = 35;  // foo()
const INDEX: u32 = 33;  // a[3]
const QUESTION: u32 = 31;  // a?
const NEG: u32 = 29;  // -a
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
const MODIFY: u32 = 7;  // p `age 32
const LOGIC_AND: u32 = 5;  // a && b
const LOGIC_OR: u32 = 3;  // a || b
const PIPELINE: u32 = 1;  // x |> $ + 1
