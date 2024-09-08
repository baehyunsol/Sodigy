use crate::{
    ArgDef,
    BranchArm,
    DottedNames,
    error::{AstError, AstErrorKind, AttributeIn, NewExpectedTokens},
    expr::{Expr, ExprKind},
    FieldKind,
    GenericDef,
    IdentWithSpan,
    let_::Let,
    MatchArm,
    ops::{
        InfixOp,
        PostfixOp,
        PrefixOp,
        call_binding_power,
        index_binding_power,
        infix_binding_power,
        path_binding_power,
        postfix_binding_power,
        prefix_binding_power,
        struct_init_binding_power,
    },
    pattern::parse_pattern_full,
    ScopeBlock,
    session::AstSession,
    stmt::{
        Attribute,
        Decorator,
        FieldDef,
        Import,
        ImportedName,
        Stmt,
        StmtKind,
        VariantDef,
        VariantKind,
    },
    StructInitDef,
    tokens::Tokens, Token, TokenKind,
    TypeDef,
    utils::{
        IntoCharError,
        format_string_into_expr,
        try_into_char,
    },
    value::ValueKind,
};
use log::{debug, info};
use sodigy_error::{ErrorContext, ExpectedToken, SodigyError};
use sodigy_intern::try_intern_short_string;
use sodigy_keyword::Keyword;
use sodigy_lex::QuoteKind;
use sodigy_parse::{Delim, Punct};
use sodigy_session::SodigySession;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

pub fn parse_stmts(tokens: &mut Tokens, session: &mut AstSession) -> Result<(), ()> {
    info!(
        "sodigy_ast::parse_stmts(), first few tokens are: {:?}",
        tokens.first_few_tokens(),
    );

    loop {
        match tokens.step() {
            Some(Token {
                kind: TokenKind::Keyword(k),
                span: keyword_span,
            }) => {
                let keyword = *k;
                let keyword_span = *keyword_span;

                match keyword {
                    Keyword::Let => {
                        match parse_let_statement(
                            tokens,
                            session,
                            true,
                            vec![],  // Attributes
                        ) {
                            Ok(l) => {
                                session.push_result(Stmt {
                                    kind: StmtKind::Let(l),
                                    span: keyword_span,
                                });
                            },
                            _ => {
                                tokens.march_until_stmt();
                                continue;
                            },
                        }
                    },
                    Keyword::Module => {
                        // 'module' IDENTIFIER ';'
                        let mod_name = match tokens.expect_ident() {
                            Ok(id) => id,
                            Err(e) => {
                                session.push_error(e);
                                tokens.march_until_stmt();
                                continue;
                            },
                        };

                        let span = keyword_span.merge(*mod_name.span());

                        if let Err(e) = tokens.consume(TokenKind::semi_colon()) {
                            session.push_error(e);
                            tokens.march_until_stmt();
                            continue;
                        }

                        session.push_result(Stmt {
                            kind: StmtKind::Module(mod_name, Uid::new_module()),
                            span,
                        });
                    },
                    Keyword::Import => {
                        match parse_import(tokens, session, keyword_span) {
                            Ok(i) => {
                                session.push_result(Stmt {
                                    kind: StmtKind::Import(i),
                                    span: keyword_span,
                                });
                            },
                            Err(()) => {
                                tokens.march_until_stmt();
                                continue;
                            },
                        }
                    },
                    unexpected_keyword => {
                        let mut e = AstError::unexpected_token(
                            Token::new_keyword(unexpected_keyword, keyword_span),
                            ExpectedToken::stmt(),
                        );

                        if unexpected_keyword == Keyword::From {
                            e.set_message(String::from("`from` comes after `import`. Try `import ... from ...;` instead of `from ... import ...;`."));
                        }

                        session.push_error(e);
                        tokens.march_until_stmt();
                        continue;
                    },
                }
            },
            Some(Token {
                kind: TokenKind::Punct(Punct::At),
                span: at_span,
            }) => {
                let at_span = *at_span;

                match parse_decorator(at_span, tokens, session) {
                    Ok((deco, span)) => {
                        session.push_result(Stmt {
                            kind: StmtKind::Decorator(deco),
                            span,
                        });
                    },
                    Err(()) => {
                        tokens.march_until_stmt();
                        continue;
                    },
                }
            },
            Some(Token {
                kind: TokenKind::DocComment(comment),
                span: doc_comment_span,
            }) => {
                let doc_comment_span = *doc_comment_span;

                session.push_result(Stmt {
                    kind: StmtKind::DocComment(*comment),
                    span: doc_comment_span,
                });
            },
            Some(unexpected_token) => {
                session.push_error(AstError::unexpected_token(
                    unexpected_token.clone(),
                    ExpectedToken::stmt(),
                ));
                tokens.march_until_stmt();
            },
            None => {
                break;
            },
        }
    }

    session.err_if_has_error()
}

type TrailingComma = bool;

pub fn parse_comma_separated_exprs(tokens: &mut Tokens, session: &mut AstSession) -> Result<(Vec<Expr>, TrailingComma), ()> {
    let mut result = vec![];
    let mut trailing_comma: TrailingComma = false;

    loop {
        if tokens.is_finished() {
            return Ok((result, trailing_comma));
        }

        result.push(parse_expr(tokens, session, 0, false, None, tokens.peek_span().unwrap())?);
        trailing_comma = false;

        match tokens.consume(TokenKind::comma()) {
            Ok(_) => {
                trailing_comma = true;
                continue;
            },
            Err(AstError {
                kind: AstErrorKind::UnexpectedEnd(_),
                ..
            }) => {
                return Ok((result, trailing_comma));
            },
            Err(e) => {
                session.push_error(e);
                return Err(());
            },
        }
    }
}

pub fn parse_comma_separated_types(tokens: &mut Tokens, session: &mut AstSession) -> Result<(Vec<TypeDef>, TrailingComma), ()> {
    let (exprs, trailing_comma) = parse_comma_separated_exprs(tokens, session)?;

    Ok((
        exprs.into_iter().map(
            |expr| TypeDef::from_expr(expr)
        ).collect(),
        trailing_comma
    ))
}

// NOTE: deciding whether to continue or not when there's an error
//       1. if there's an error inside paren/brace/bracket, it continues parsing the tokens after the paren/brace/bracket
//          - it's sure that the error will not affect the tokens outside paren/brace/bracket
//       2. otherwise, it halts immediately
//          - it doesn't know where the error ends
// pratt parsing
// TODO: right-associativity
pub fn parse_expr(
    tokens: &mut Tokens,
    session: &mut AstSession,
    min_bp: u32,

    // the behavior of this flag depends on the first token
    // 1. if it's not an expr at all, it does nothing
    // 2. if it's a valid expr, it either succeeds or dies
    do_nothing_when_failed: bool,

    error_context: Option<ErrorContext>,

    // when `tokens` is empty, it uses this span for the error message
    parent_span: SpanRange,
) -> Result<Expr, ()> {
    debug!(
        "sodigy_ast::parse_expr(), first few tokens are: {:?}, min_bp: {min_bp}, error_context: {error_context:?}",
        tokens.first_few_tokens(),
    );

    let mut lhs = match tokens.step() {
        Some(Token {
            kind: TokenKind::Punct(p),
            span,
        }) => {
            let punct = *p;
            let prefix_op_span = *span;

            match PrefixOp::try_from(punct) {
                Ok(op) => {
                    let bp = prefix_binding_power(op);
                    let rhs = parse_expr(tokens, session, bp, false, error_context, prefix_op_span)?;

                    Expr {
                        kind: ExprKind::PrefixOp(op, Box::new(rhs)),
                        span: prefix_op_span,
                    }
                },
                Err(()) if do_nothing_when_failed => {
                    tokens.backward().unwrap();
                    return Err(());
                },
                Err(()) => {
                    let message = if punct == Punct::DotDot {
                        String::from("`..` is not a valid prefix operator. If you want its lhs to be 0, specify it. Like `0..`")
                    }

                    else {
                        format!("`{punct}` is not a valid prefix operator.")
                    };

                    session.push_error(AstError::unexpected_token(
                        Token::new_punct(punct, prefix_op_span),
                        ExpectedToken::expr(),
                    ).set_message(
                        message
                    ).try_set_error_context(
                        error_context,
                    ).to_owned());

                    return Err(());
                },
            }
        },
        Some(Token {
            kind: TokenKind::Keyword(Keyword::If),
            span,
        }) => {
            let span = *span;
            tokens.backward().unwrap();
            let mut branch_arms = vec![];
            let mut else_span = None;

            loop {
                let branch_arm = parse_branch_arm(tokens, session, span, else_span)?;
                let has_cond = branch_arm.cond.is_some();

                branch_arms.push(branch_arm);

                if !tokens.is_curr_token(TokenKind::Keyword(Keyword::Else)) || !has_cond {
                    break;
                }

                // step `else`
                else_span = Some(tokens.step().unwrap().span);
            }

            Expr {
                kind: ExprKind::Branch(branch_arms),
                span,
            }
        },
        Some(Token {
            kind: TokenKind::Keyword(Keyword::Match),
            span,
        }) => {
            let span = *span;
            let value = parse_expr(tokens, session, 0, false, error_context, span)?;
            let group_span = tokens.peek_span();

            match tokens.expect_group(Delim::Brace) {
                Ok(match_body_tokens) => {
                    let group_span = group_span.unwrap();
                    let mut match_body_tokens = match_body_tokens.to_vec();
                    let mut match_body_tokens = Tokens::from_vec(&mut match_body_tokens);
                    let last_token_span = match_body_tokens.span_end().unwrap_or(group_span);

                    let arms = if let Ok(arms) = parse_match_body(&mut match_body_tokens, session, group_span) {
                        arms
                    } else {
                        vec![]
                    };

                    Expr {
                        kind: ExprKind::Match {
                            value: Box::new(value),
                            arms,
                            is_lowered_from_if_pattern: false,
                        },
                        span: span.merge(last_token_span),
                    }
                },
                Err(mut e) => {
                    session.push_error(
                        e.set_error_context(
                            ErrorContext::ParsingMatchBody,
                        ).to_owned()
                    );
                    return Err(());
                },
            }
        },
        Some(Token {
            kind: TokenKind::Keyword(k),
            span,
        }) => {
            if do_nothing_when_failed {
                tokens.backward().unwrap();
                return Err(());
            }

            else {
                session.push_error(AstError::unexpected_token(
                    Token::new_keyword(*k, *span),
                    ExpectedToken::expr(),
                ).try_set_error_context(
                    error_context,
                ).to_owned());
                return Err(());
            }
        },
        Some(curr @ Token {
            kind: TokenKind::Group { delim, tokens, prefix },
            span,
        }) => {
            let delim = *delim;
            let span = *span;
            let prefix = *prefix;
            let mut tokens = tokens.to_vec();

            if prefix == b'\\' {
                if delim == Delim::Brace {
                    let mut tokens = Tokens::from_vec(&mut tokens);
                    tokens.set_span_end(span.last_char());

                    let (args, value) = if let Ok((args, value)) = parse_lambda_body(&mut tokens, session, span) {
                        (args, value)
                    } else {
                        (
                            vec![],
                            Expr {
                                kind: ExprKind::Error,
                                span,  // nobody cares about its span
                            },
                        )
                    };

                    Expr {
                        kind: ExprKind::Value(ValueKind::Lambda {
                            args,
                            value: Box::new(value),
                            uid: Uid::new_lambda(),

                            return_ty: None,  // users cannot annotate return_ty of a lambda
                            lowered_from_scoped_let: false,
                        }),
                        span,
                    }
                }

                else {
                    let mut curr_token = curr.clone();
                    curr_token.remove_prefix();

                    session.push_error(AstError::unexpected_token(
                        curr_token,
                        ExpectedToken::specific(TokenKind::Group { delim: Delim::Brace, tokens: vec![], prefix: b'\0' }),
                    ).set_message(
                        String::from("If you're to use a lambda function, use curly braces.")
                    ).try_set_error_context(
                        error_context,
                    ).to_owned());
                    return Err(());
                }
            }

            else {
                match delim {
                    Delim::Paren => {
                        let mut tokens = Tokens::from_vec(&mut tokens);
                        tokens.set_span_end(span.last_char());

                        match parse_comma_separated_exprs(&mut tokens, session) {
                            Ok((elems, has_trailing_comma)) if !has_trailing_comma && elems.len() == 1 => {
                                Expr {
                                    kind: ExprKind::Parenthesis(Box::new(elems[0].clone())),
                                    span,
                                }
                            },
                            Ok((elems, _)) => {
                                Expr {
                                    kind: ExprKind::Value(ValueKind::Tuple(elems)),
                                    span,
                                }
                            },
                            Err(()) => {
                                Expr {
                                    kind: ExprKind::Error,
                                    span,
                                }
                            },
                        }
                    },
                    Delim::Bracket => {
                        let mut tokens = Tokens::from_vec(&mut tokens);
                        tokens.set_span_end(span.last_char());

                        let elems = if let Ok((elems, _)) = parse_comma_separated_exprs(&mut tokens, session) {
                            elems
                        } else {
                            vec![]
                        };

                        Expr {
                            kind: ExprKind::Value(ValueKind::List(elems)),
                            span,
                        }
                    },
                    Delim::Brace => {
                        let mut tokens = Tokens::from_vec(&mut tokens);
                        tokens.set_span_end(span.last_char());

                        if let Ok(scope) = parse_scope_block(&mut tokens, session, span) {
                            Expr {
                                kind: ExprKind::Value(ValueKind::Scope {
                                    scope,
                                    uid: Uid::new_scope(),
                                }),
                                span,
                            }
                        } else {
                            Expr {
                                kind: ExprKind::Error,
                                span,
                            }
                        }
                    },
                }
            }
        },
        Some(Token {
            kind: TokenKind::Identifier(id),
            span,
        }) => {
            let id = *id;
            let span = *span;

            Expr {
                kind: ExprKind::Value(ValueKind::Identifier(id)),
                span,
            }
        },
        Some(Token {
            kind: TokenKind::Number(n),
            span,
        }) => {
            let n = *n;
            let span = *span;

            Expr {
                kind: ExprKind::Value(ValueKind::Number(n)),
                span,
            }
        },
        Some(Token {
            kind: TokenKind::String { kind, content, is_binary },
            span,
        }) => {
            let span = *span;
            let is_binary = *is_binary;

            match *kind {
                QuoteKind::Double => Expr {
                    kind: ExprKind::Value(ValueKind::String {
                        content: *content,
                        is_binary,
                    }),
                    span,
                },
                QuoteKind::Single => if is_binary {
                    // There are no binary chars, because `Char`s in Sodigy are just integers
                    session.push_error(AstError::binary_char(span).try_set_error_context(
                        error_context,
                    ).to_owned());
                    return Err(());
                }

                else if let Some((length, bytes)) = content.try_unwrap_short_string() {
                    match try_into_char(&bytes[0..(length as usize)]) {
                        Ok(c) => Expr {
                            kind: ExprKind::Value(ValueKind::Char(c)),
                            span,
                        },
                        Err(e) => {
                            session.push_error(
                                e.into_ast_error(span).try_set_error_context(
                                    error_context,
                                ).to_owned()
                            );
                            return Err(());
                        },
                    }
                }

                else {
                    session.push_error(
                        IntoCharError::TooLong.into_ast_error(span).try_set_error_context(
                            error_context,
                        ).to_owned()
                    );
                    return Err(());
                },
            }
        },
        Some(Token {
            kind: TokenKind::DocComment(_),
            span,
        }) => {
            if do_nothing_when_failed {
                tokens.backward().unwrap();
                return Err(());
            }

            else {
                session.push_error(AstError::unexpected_token(
                    Token::new_doc_comment(
                        try_intern_short_string(b"...").unwrap(),
                        *span,
                    ),
                    ExpectedToken::expr(),
                ).try_set_error_context(
                    error_context,
                ).to_owned());
                return Err(());
            }
        },
        Some(Token {
            kind: TokenKind::FormattedString(elems),
            span,
        }) => {
            let span = *span;
            let elems: Vec<_> = elems.iter().filter_map(
                // string literals in a formatted string share spans
                |elem| format_string_into_expr(elem, span, session).ok()
            ).collect();

            session.err_if_has_error()?;

            Expr {
                kind: ExprKind::Value(ValueKind::Format(
                    elems
                )),
                span,
            }
        },
        // it has to be lowered before this stage
        Some(Token {
            kind: TokenKind::Macro { .. },
            ..
        }) => unreachable!(),
        None => {
            if do_nothing_when_failed {
                return Err(());
            }

            else {
                session.push_error(AstError::unexpected_end(
                    tokens.span_end().unwrap_or(parent_span.last_char()),
                    ExpectedToken::expr(),
                ).try_set_error_context(
                    error_context,
                ).to_owned());
                return Err(());
            }
        },
    };

    loop {
        match tokens.step() {
            Some(Token {
                kind: TokenKind::Punct(p),
                span,
            }) => {
                let punct = *p;
                let punct_span = *span;

                match PostfixOp::try_from(punct) {
                    // `..` can both be infix and postfix!
                    Ok(op @ PostfixOp::Range) => {
                        let bp = postfix_binding_power(op);

                        if bp < min_bp {
                            // parse this op later
                            tokens.backward().unwrap();
                            break;
                        }

                        match parse_expr(tokens, session, bp, true, error_context, punct_span) {
                            Ok(rhs) => {
                                lhs = Expr {
                                    kind: ExprKind::InfixOp(
                                        InfixOp::try_from(op).unwrap(),
                                        Box::new(lhs),
                                        Box::new(rhs),
                                    ),
                                    span: punct_span,
                                };
                            },
                            Err(_) => {
                                lhs = Expr {
                                    kind: ExprKind::PostfixOp(op, Box::new(lhs)),
                                    span: punct_span,
                                };
                            },
                        }
                        continue;
                    },
                    Ok(op) => {
                        let bp = postfix_binding_power(op);

                        if bp < min_bp {
                            // parse this op later
                            tokens.backward().unwrap();
                            break;
                        }

                        lhs = Expr {
                            kind: ExprKind::PostfixOp(op, Box::new(lhs)),
                            span: punct_span,
                        };
                        continue;
                    },
                    Err(_) => {
                        // let's try infix
                    }
                }

                // path operator
                if punct == Punct::Dot {
                    let (l_bp, _) = path_binding_power();

                    if l_bp < min_bp {
                        tokens.backward().unwrap();
                        break;
                    }

                    let rhs = match tokens.expect_ident() {
                        Ok(id) => id,
                        Err(mut e) => {
                            e.try_set_error_context(error_context);

                            if matches!(e.kind, AstErrorKind::UnexpectedToken(..)) {
                                e.set_message(String::from("A name of a field must be an identifier."));
                            }

                            else if matches!(e.kind, AstErrorKind::UnexpectedEnd(_)) {
                                e.set_message(String::from("Please provide the name of a field."));
                            }

                            session.push_error(e);
                            return Err(());
                        },
                    };

                    lhs = Expr {
                        kind: ExprKind::Field { pre: Box::new(lhs), post: FieldKind::Named(rhs) },
                        span: punct_span,
                    };
                    continue;
                }

                match InfixOp::try_from(punct) {
                    Ok(op) => {
                        let (l_bp, r_bp) = infix_binding_power(op);

                        if l_bp < min_bp {
                            // parse this op later
                            tokens.backward().unwrap();
                            break;
                        }

                        let rhs = if let Ok(expr) = parse_expr(tokens, session, r_bp, false, error_context, punct_span) {
                            expr
                        } else {
                            return Err(());
                        };

                        lhs = Expr {
                            kind: ExprKind::InfixOp(op, Box::new(lhs), Box::new(rhs)),
                            span: punct_span,
                        };
                        continue;
                    },
                    Err(_) => {
                        tokens.backward().unwrap();
                        break;
                    }
                }
            },
            Some(curr @ Token {
                kind: TokenKind::Group { delim, tokens: inner_tokens, prefix },
                span,
            }) => {
                let span = *span;
                let prefix = *prefix;

                if prefix == b'\\' {
                    session.push_error(AstError::unexpected_token(
                        curr.clone(),
                        ExpectedToken::post(),
                    ).set_message(
                        String::from("Try remove `\\`.")
                    ).try_set_error_context(
                        error_context,
                    ).to_owned());
                    return Err(());
                }

                else {
                    let mut inner_tokens = inner_tokens.to_vec();

                    match delim {
                        Delim::Bracket => {
                            let (l_bp, _) = index_binding_power();

                            if l_bp < min_bp {
                                // parse this op later
                                tokens.backward().unwrap();
                                break;
                            }

                            let mut index_tokens = Tokens::from_vec(&mut inner_tokens);
                            index_tokens.set_span_end(span.last_char());

                            let rhs = if let Ok(expr) = parse_expr(&mut index_tokens, session, 0, false, error_context, span) {
                                expr
                            } else {
                                Expr {
                                    kind: ExprKind::Error,
                                    span,
                                }
                            };

                            if !index_tokens.is_finished() {
                                session.push_error(AstError::unexpected_token(
                                    index_tokens.peek().unwrap().clone(),
                                    ExpectedToken::nothing(),
                                ).try_set_error_context(
                                    error_context,
                                ).to_owned());
                                return Err(());
                            }

                            lhs = Expr {
                                kind: ExprKind::InfixOp(InfixOp::Index, Box::new(lhs), Box::new(rhs)),
                                span,
                            };
                            continue;
                        },
                        Delim::Paren => {
                            let (l_bp, _) = call_binding_power();

                            if l_bp < min_bp {
                                // parse this op later
                                tokens.backward().unwrap();
                                break;
                            }

                            let mut index_tokens = Tokens::from_vec(&mut inner_tokens);
                            index_tokens.set_span_end(span.last_char());

                            let args = if let Ok((args, _)) = parse_comma_separated_exprs(&mut index_tokens, session) {
                                args
                            } else {
                                vec![]
                            };

                            lhs = Expr {
                                kind: ExprKind::Call {
                                    func: Box::new(lhs),
                                    args,
                                },
                                span,
                            };
                            continue;
                        },

                        // there are multiple cases:
                        // `if foo { 3 }` is valid, but it's not a struct initialization
                        // `if foo { bar: 3 }` is valid, and is a struct initialization
                        Delim::Brace => {
                            let (l_bp, _) = struct_init_binding_power();

                            if l_bp < min_bp {
                                tokens.backward().unwrap();
                                break;
                            }

                            let mut struct_init_tokens = Tokens::from_vec(&mut inner_tokens);
                            struct_init_tokens.set_span_end(span.last_char());

                            match try_parse_struct_init(&mut struct_init_tokens, session) {
                                Some(Ok(s)) => {
                                    lhs = Expr {
                                        kind: ExprKind::StructInit {
                                            struct_: Box::new(lhs),
                                            fields: s,
                                        },
                                        span,
                                    };
                                    continue;
                                },

                                // it's a struct initialization,
                                // but there's a synax error
                                Some(Err(_)) => {
                                    return Err(());
                                },

                                // not a struct initialization
                                None => {
                                    tokens.backward().unwrap();
                                    break;
                                },
                            }
                        },
                    }
                }
            },
            _ => {
                tokens.backward().unwrap();
                break;
            }
        }
    }

    Ok(lhs)
}

pub fn parse_type_def(
    tokens: &mut Tokens,
    session: &mut AstSession,

    // when `tokens` is empty, it uses this span for the error message
    parent_span: SpanRange,
) -> Result<TypeDef, ()> {
    // this branch doesn't do anything but provides better error message
    if tokens.is_finished() {
        session.push_error(AstError::unexpected_end(
            tokens.span_end().unwrap_or(parent_span),
            ExpectedToken::ty(),
        ));
        return Err(());
    }

    match parse_expr(
        tokens,
        session,
        0,
        false,
        Some(ErrorContext::ParsingTypeAnnotation),
        parent_span,
    ) {
        Ok(expr) => Ok(TypeDef::from_expr(expr)),
        Err(_) => {
            // TODO: there must be users who say `Option<T>` instead of `Option(T)`
            //       and this is where those cases would generate an error
            //       I want to warn them
            //       see samples/errors/generic_err.sdg
            Err(())
        },
    }
}

// this function allows a trailing comma and args without type annotations
// it's your responsibility to check type annotations
fn parse_arg_defs(tokens: &mut Tokens, session: &mut AstSession) -> Result<Vec<ArgDef>, ()> {
    let mut args = vec![];
    let mut attributes = vec![];

    loop {
        if tokens.is_finished() {
            if !attributes.is_empty() {
                session.push_error(AstError::stranded_attribute(attributes, AttributeIn::FuncArg));
                return Err(());
            }

            return Ok(args);
        }

        if tokens.is_curr_token(TokenKind::Punct(Punct::At)) {
            let at_span = tokens.step().unwrap().span;

            let (deco, _) = parse_decorator(
                at_span, tokens, session,
            )?;

            attributes.push(Attribute::Decorator(deco));
            continue;
        }

        if tokens.is_curr_token_doc_comment() {
            let curr_span = tokens.peek_span().unwrap();

            attributes.push(Attribute::DocComment(
                IdentWithSpan::new(
                    tokens.expect_doc_comment().unwrap(),
                    curr_span,
                )
            ));

            tokens.step().unwrap();
            continue;
        }

        let arg_name = match tokens.expect_ident() {
            Ok(id) => id,
            Err(e) => {
                session.push_error(e);
                return Err(());
            },
        };

        let has_question_mark = if tokens.is_curr_token(TokenKind::Punct(Punct::QuestionMark)) {
            tokens.step().unwrap();

            true
        } else {
            false
        };

        let mut arg_type = None;

        match tokens.step() {
            Some(Token {
                kind: TokenKind::Punct(Punct::Colon),
                span: colon_span,
            }) => {
                let colon_span = *colon_span;

                arg_type = Some(parse_type_def(
                    tokens,
                    session,
                    colon_span,
                )?);
            },
            Some(Token {
                kind: TokenKind::Punct(Punct::Comma),
                ..
            }) => {
                args.push(ArgDef {
                    name: arg_name,
                    ty: arg_type,
                    has_question_mark,
                    attributes: attributes.clone(),
                });

                attributes.clear();

                continue;
            },
            Some(token) => {
                let mut e = AstError::unexpected_token(
                    token.clone(),
                    ExpectedToken::comma_or_colon(),
                );

                if let Token {
                    kind: TokenKind::Punct(Punct::Gt),
                    ..
                } = token {
                    e.set_message(String::from(
                        "TODO: a kind error message telling the user that it must be parentheses, not angle brackets",
                    ));
                }

                session.push_error(e);
                return Err(());
            },
            None => {
                args.push(ArgDef {
                    name: arg_name,
                    ty: arg_type,
                    has_question_mark,
                    attributes: attributes.clone(),
                });
                return Ok(args);
            },
        }

        match tokens.consume(TokenKind::comma()) {
            Ok(()) => {
                args.push(ArgDef {
                    name: arg_name,
                    ty: arg_type,
                    has_question_mark,
                    attributes: attributes.clone(),
                });

                attributes.clear();
            },
            Err(AstError {
                kind: AstErrorKind::UnexpectedEnd(_),
                ..
            }) => {
                args.push(ArgDef {
                    name: arg_name,
                    ty: arg_type,
                    has_question_mark,
                    attributes: attributes.clone(),
                });
                return Ok(args);
            },
            Err(e) => {
                session.push_error(e);
                return Err(());
            },
        }
    }
}

fn parse_scope_block(
    tokens: &mut Tokens,
    session: &mut AstSession,
    span: SpanRange,
) -> Result<ScopeBlock, ()> {
    if tokens.is_finished() {
        session.push_error(AstError::empty_scope_block(span));
        return Err(());
    }

    let mut lets = vec![];
    let mut attributes = vec![];
    let mut has_error = false;

    loop {
        if tokens.is_curr_token(TokenKind::Keyword(Keyword::Let)) {
            tokens.step().unwrap();
            lets.push(parse_let_statement(tokens, session, false, attributes.clone())?);
            attributes.clear();
        }

        else if tokens.is_curr_token_doc_comment() {
            let curr_span = tokens.peek_span().unwrap();

            attributes.push(Attribute::DocComment(
                IdentWithSpan::new(
                    tokens.expect_doc_comment().unwrap(),
                    curr_span,
                )
            ));

            tokens.step().unwrap();
        }

        // we have to make sure that `@` is always a decorator, not an expression
        // for now, this holds because all the macros are expanded before the AST pass
        else if tokens.is_curr_token(TokenKind::Punct(Punct::At)) {
            let at_span = tokens.step().unwrap().span;

            let (deco, _) = parse_decorator(
                at_span, tokens, session,
            )?;

            attributes.push(Attribute::Decorator(deco));
        }

        else {
            if !attributes.is_empty() {
                session.push_error(AstError::stranded_attribute(
                    attributes,
                    AttributeIn::ScopedLet,
                ));
                has_error = true;
            }

            break;
        }
    }

    let value = parse_expr(
        tokens,
        session,
        0,
        false,
        Some(ErrorContext::ParsingScopeBlock),
        span,
    )?;

    if !tokens.is_finished() {
        let curr_token = tokens.peek().unwrap();
        let mut e = AstError::unexpected_token(
            curr_token.clone(),
            ExpectedToken::nothing(),
        );

        if curr_token.kind == TokenKind::semi_colon() {
            e.set_message("`;`s are used to separate statements, not the value of a block. Try remove this `;`.".to_string());
        }

        session.push_error(e);

        return Err(());
    }

    if has_error {
        return Err(());
    }

    Ok(ScopeBlock { lets, value: Box::new(value) })
}

// `\{x, y, x + y}`
// `\{x: Int, y: Int, x + y}`
// `\{foo()}`
// `\{x, y}`   -> could be valid if `y` is defined somewhere
// `\{x}`      -> could be valid if `x` is defined somewhere
// `\{x, y,}`  -> can never be valid
fn parse_lambda_body(tokens: &mut Tokens, session: &mut AstSession, span: SpanRange) -> Result<(Vec<ArgDef>, Expr), ()> {
    let mut args = vec![];

    loop {
        match tokens.step() {
            Some(Token {
                kind: TokenKind::Identifier(id),
                span,
            }) => {
                let id = *id;
                let span = *span;

                /* expr or param, but not sure yet */
                let curr_arg = IdentWithSpan::new(id, span);
                let has_question_mark = if tokens.is_curr_token(TokenKind::Punct(Punct::QuestionMark)) {
                    tokens.step().unwrap();

                    true
                } else {
                    false
                };

                match tokens.step() {
                    Some(Token {
                        kind: TokenKind::Punct(Punct::Colon),
                        span: colon_span,
                    }) => {
                        let colon_span = *colon_span;

                        /* the last ident is an arg */
                        /* now it's a type annotation */
                        let ty_anno = parse_type_def(
                            tokens,
                            session,
                            colon_span,
                        )?;

                        args.push(ArgDef {
                            name: curr_arg,
                            ty: Some(ty_anno),
                            has_question_mark,

                            // attrs for lambda args is not implemented yet
                            attributes: vec![],
                        });

                        if let Err(e) = tokens.consume(TokenKind::comma()) {
                            session.push_error(e);
                            return Err(());
                        }
                    },
                    Some(Token {
                        kind: TokenKind::Punct(Punct::Comma),
                        ..
                    }) => {
                        /* the last ident is an arg */
                        args.push(ArgDef {
                            name: curr_arg,
                            ty: None,
                            has_question_mark,

                            // attrs for lambda args is not implemented yet
                            attributes: vec![],
                        });
                        continue;
                    },
                    Some(_) => {
                        /* the last ident is an expr */
                        tokens.backward().unwrap();
                        tokens.backward().unwrap();

                        if has_question_mark {
                            tokens.backward().unwrap();
                        }

                        let last_span = tokens.peek_span().unwrap();

                        let expr = parse_expr(
                            tokens,
                            session,
                            0,
                            false,
                            Some(ErrorContext::ParsingLambdaBody),
                            last_span,
                        )?;

                        if !tokens.is_finished() {
                            session.push_error(AstError::unexpected_token(
                                tokens.peek().unwrap().clone(),
                                ExpectedToken::nothing(),
                            ));
                        }

                        return Ok((args, expr));
                    },
                    None => {
                        let expr = Expr {
                            kind: ExprKind::Value(ValueKind::Identifier(id)),
                            span,
                        };

                        return Ok((args, expr));
                    },
                }
            },
            Some(_) => {
                /* expr */
                tokens.backward().unwrap();
                let last_span = tokens.peek_span().unwrap();

                let expr = parse_expr(
                    tokens,
                    session,
                    0,
                    false,
                    Some(ErrorContext::ParsingLambdaBody),
                    last_span,
                )?;

                if !tokens.is_finished() {
                    session.push_error(AstError::unexpected_token(
                        tokens.peek().unwrap().clone(),
                        ExpectedToken::nothing(),
                    ));
                }

                return Ok((args, expr));
            },
            None => {
                /* unexpected end */
                session.push_error(AstError::unexpected_end(
                    span,
                    ExpectedToken::expr(),
                ));
                return Err(());
            },
        }
    }
}

fn parse_match_body(tokens: &mut Tokens, session: &mut AstSession, span: SpanRange) -> Result<Vec<MatchArm>, ()> {
    let mut arms = vec![];

    loop {
        if tokens.is_finished() {
            if arms.is_empty() {
                session.push_error(
                    AstError::empty_match_body(span).set_error_context(
                        ErrorContext::ParsingMatchBody
                    ).to_owned()
                );
                return Err(());
            }

            else {
                return Ok(arms);
            }
        }

        let pattern = parse_pattern_full(tokens, session)?;
        let mut guard = None;
        let rarrow_span;

        match tokens.step() {
            Some(Token {
                kind: TokenKind::Punct(Punct::RArrow),
                span,
            }) => {
                rarrow_span = Some(*span);
            },
            Some(Token {
                kind: TokenKind::Keyword(Keyword::If),
                span: if_span,
            }) => {
                let if_span = *if_span;
                guard = Some(parse_expr(
                    tokens,
                    session,
                    0,
                    false,
                    Some(ErrorContext::ParsingMatchBody),
                    if_span,
                )?);
                rarrow_span = tokens.peek_span();

                if let Err(mut e) = tokens.consume(TokenKind::r_arrow()) {
                    session.push_error(e.set_error_context(
                        ErrorContext::ParsingMatchBody
                    ).to_owned());
                    return Err(());
                }
            },
            Some(token) => {
                let mut e = AstError::unexpected_token(
                    token.clone(),
                    ExpectedToken::guard_or_arrow(),
                );
                e.set_error_context(ErrorContext::ParsingMatchBody);

                if let TokenKind::Punct(Punct::At) = &token.kind {
                    e.set_message(String::from("To bind a name to a pattern, the name must come before the pattern, not after it."));
                }

                session.push_error(e);

                // check typo: `->` instead of `=>`
                if token.kind == TokenKind::Punct(Punct::Sub) {
                    let token = token.clone();

                    if tokens.is_curr_token(TokenKind::gt()) {
                        session.pop_error().unwrap();

                        session.push_error(AstError::unexpected_token(
                            token,
                            ExpectedToken::specific(TokenKind::r_arrow()),
                        ).set_error_context(
                            ErrorContext::ParsingMatchBody
                        ).set_message(
                            String::from("Use `=>` instead of `->`.")
                        ).to_owned());
                        return Err(());
                    }
                }

                return Err(());
            },
            None => {
                session.push_error(AstError::unexpected_end(
                    span,
                    ExpectedToken::guard_or_arrow(),
                ).set_error_context(
                    ErrorContext::ParsingMatchBody
                ).to_owned());
                return Err(());
            },
        }

        let value = parse_expr(
            tokens,
            session,
            0,
            false,
            Some(ErrorContext::ParsingMatchBody),
            rarrow_span.unwrap(),
        )?;

        arms.push(MatchArm {
            pattern, value, guard,
            uid: Uid::new_match_arm(),
        });

        match tokens.consume(TokenKind::comma()) {
            Err(AstError {
                kind: AstErrorKind::UnexpectedEnd(_),
                ..
            }) => {
                return Ok(arms);
            },
            Err(mut e) => {
                session.push_error(e.set_error_context(
                    ErrorContext::ParsingMatchBody
                ).to_owned());
                return Err(());
            },
            _ => {},
        }
    }
}

// it expects either `if` or `{ ... }`
fn parse_branch_arm(
    tokens: &mut Tokens,
    session: &mut AstSession,

    // when `tokens` is empty, it uses this span for the error message
    parent_span: SpanRange,
    else_span: Option<SpanRange>,
) -> Result<BranchArm, ()> {
    match tokens.step() {
        Some(Token {
            kind: TokenKind::Keyword(Keyword::If),
            span: if_span,
        }) => {
            let mut head_span = if let Some(span) = else_span {
                span.merge(*if_span)
            } else {
                *if_span
            };
            let mut pattern_bind = None;

            if tokens.is_curr_token(TokenKind::Keyword(Keyword::Pattern)) {
                head_span = head_span.merge(tokens.step().unwrap().span);

                let pat = parse_pattern_full(tokens, session)?;

                if let Err(mut e) = tokens.consume(TokenKind::assign()) {
                    session.push_error(
                        e.set_error_context(
                            ErrorContext::ParsingBranchCondition
                        ).to_owned()
                    );
                    return Err(());
                }

                pattern_bind = Some(pat);
            }

            let cond = parse_expr(tokens, session, 0, false, Some(ErrorContext::ParsingBranchCondition), head_span)?;
            let span = tokens.peek_span();

            match tokens.expect_group(Delim::Brace) {
                Ok(tokens) => {
                    let mut val_tokens = tokens.to_vec();
                    let mut val_tokens = Tokens::from_vec(&mut val_tokens);
                    let span = span.unwrap();

                    let scope = parse_scope_block(&mut val_tokens, session, span)?;
                    let mut value = Expr {
                        kind: ExprKind::Value(ValueKind::Scope {
                            scope,
                            uid: Uid::new_scope(),
                        }),
                        span,
                    };

                    value.peel_unnecessary_brace();

                    Ok(BranchArm {
                        cond: Some(cond),
                        pattern_bind,
                        value,
                        span: head_span,
                    })
                },
                Err(mut e) => {
                    if cond.starts_with_curly_brace() {
                        e.set_message("It seems like you're missing a condition of a branch.".to_string());
                    }

                    session.push_error(e);
                    return Err(());
                },
            }
        },
        Some(Token {
            kind: TokenKind::Group {
                delim: Delim::Brace,
                tokens: val_tokens,
                prefix: b'\0',
            },
            span,
        }) => {
            let span = *span;
            let mut val_tokens = val_tokens.to_vec();
            let mut val_tokens = Tokens::from_vec(&mut val_tokens);
            val_tokens.set_span_end(span.last_char());

            let scope = parse_scope_block(&mut val_tokens, session, span)?;
            let mut value = Expr {
                kind: ExprKind::Value(ValueKind::Scope {
                    scope,
                    uid: Uid::new_scope(),
                }),
                span,
            };

            value.peel_unnecessary_brace();

            Ok(BranchArm {
                cond: None,
                pattern_bind: None,
                value,
                span: else_span.unwrap(),
            })
        },
        Some(token) => {
            session.push_error(AstError::unexpected_token(
                token.clone(),
                ExpectedToken::if_or_brace(),
            ));

            Err(())
        },
        None => {
            session.push_error(AstError::unexpected_end(
                tokens.span_end().unwrap_or(parent_span.last_char()),
                ExpectedToken::if_or_brace(),
            ));

            Err(())
        },
    }
}

// `@` is already consumed
fn parse_decorator(at_span: SpanRange, tokens: &mut Tokens, session: &mut AstSession) -> Result<(Decorator, SpanRange), ()> {
    let mut span = at_span;
    let mut names = vec![];

    names.push(
        match tokens.expect_ident() {
            Ok(id) => {
                span = span.merge(*id.span());
                id
            },
            Err(e) => {
                session.push_error(e);
                return Err(());
            },
        }
    );

    while let Ok(()) = tokens.consume(TokenKind::dot()) {
        names.push(
            match tokens.expect_ident() {
                Ok(id) => {
                    span = span.merge(*id.span());
                    id
                },
                Err(e) => {
                    session.push_error(e);
                    return Err(());
                },
            }
        );
    }

    let paren_span = tokens.peek_span();

    let args = match tokens.expect_group(Delim::Paren) {
        Ok(mut arg_tokens) => {
            let mut arg_tokens = Tokens::from_vec(&mut arg_tokens);

            let args = match parse_comma_separated_exprs(&mut arg_tokens, session) {
                Ok((args, _)) => {
                    span = span.merge(paren_span.unwrap());
                    args
                },
                Err(()) => {
                    return Err(());
                },
            };

            Some(args)
        },
        _ => None,
    };

    Ok((Decorator { name: names, args }, span))
}

// `import` is already consumed
fn parse_import(tokens: &mut Tokens, session: &mut AstSession, keyword_span: SpanRange) -> Result<Import, ()> {
    let mut imported_names = vec![];

    loop {
        let mut alias = None;
        let name = parse_dotted_names(tokens, session, Some(ErrorContext::ParsingImportStatement))?;

        if tokens.is_curr_token(TokenKind::Keyword(Keyword::As)) {
            tokens.step().unwrap();

            match tokens.expect_ident() {
                Ok(id) => {
                    alias = Some(id);
                },
                Err(e) => {
                    session.push_error(e);
                    return Err(());
                },
            }
        }

        imported_names.push(ImportedName {
            name,
            alias,
        });

        match tokens.step() {
            Some(Token {
                kind: TokenKind::Punct(Punct::Comma),
                ..
            }) => {
                continue;
            },
            Some(Token {
                kind: TokenKind::Keyword(Keyword::From),
                ..
            }) => {
                let fr = parse_dotted_names(tokens, session, Some(ErrorContext::ParsingImportStatement))?;

                if let Err(mut e) = tokens.consume(TokenKind::semi_colon()) {
                    session.push_error(
                        e.set_error_context(
                            ErrorContext::ParsingImportStatement
                        ).to_owned()
                    );
                    return Err(());
                }

                return Ok(Import {
                    names: imported_names,
                    from: Some(fr),
                });
            },
            Some(Token {
                kind: TokenKind::Punct(Punct::SemiColon),
                ..
            }) => {
                return Ok(Import {
                    names: imported_names,
                    from: None,
                });
            },
            Some(token) => {
                session.push_error(AstError::unexpected_token(
                    token.clone(),
                    ExpectedToken::comma_semicolon_dot_or_from(),
                ));
                return Err(());
            },
            None => {
                session.push_error(AstError::unexpected_end(
                    tokens.span_end().unwrap_or(keyword_span),
                    ExpectedToken::comma_semicolon_dot_or_from(),
                ));
                return Err(());
            },
        }
    }
}

// tokens inside `{ ... }`
fn parse_struct_body(tokens: &mut Tokens, session: &mut AstSession, group_span: SpanRange) -> Result<Vec<FieldDef>, ()> {
    let mut fields = vec![];

    loop {
        if tokens.is_finished() {
            // A struct cannot be empty. See the comments in `try_parse_struct_init`
            if fields.is_empty() {
                session.push_error(AstError::empty_struct_body(group_span));
                return Err(());
            }

            else {
                return Ok(fields);
            }
        }

        let mut attributes = vec![];

        loop {
            if tokens.is_curr_token(TokenKind::Punct(Punct::At)) {
                let at_span = tokens.step().unwrap().span;

                let (deco, _) = parse_decorator(
                    at_span, tokens, session,
                )?;

                attributes.push(Attribute::Decorator(deco));
                continue;
            }

            if tokens.is_curr_token_doc_comment() {
                let curr_span = tokens.peek_span().unwrap();

                attributes.push(Attribute::DocComment(
                    IdentWithSpan::new(
                        tokens.expect_doc_comment().unwrap(),
                        curr_span,
                    )
                ));

                continue;
            }

            break;
        }

        let field_name = match tokens.expect_ident() {
            Ok(id) => id,
            Err(mut e) => {
                session.push_error(e.set_error_context(
                    ErrorContext::ParsingStructBody
                ).to_owned());
                return Err(());
            },
        };

        let colon_span = tokens.peek_span();

        if let Err(mut e) = tokens.consume(TokenKind::colon()) {
            session.push_error(e.set_error_context(
                ErrorContext::ParsingStructBody
            ).to_owned());
            return Err(());
        }

        let field_ty = parse_type_def(
            tokens,
            session,
            colon_span.unwrap(),
        )?;

        fields.push(FieldDef {
            name: field_name,
            ty: field_ty,
            attributes,
        });

        match tokens.consume(TokenKind::comma()) {
            Err(AstError {
                kind: AstErrorKind::UnexpectedEnd(_),
                ..
            }) => {
                return Ok(fields);
            },
            Err(mut e) => {
                session.push_error(e.set_error_context(
                    ErrorContext::ParsingStructBody
                ).to_owned());

                return Err(());
            },
            Ok(()) => {},
        }
    }
}

// tokens inside `{ ... }`
fn parse_enum_body(tokens: &mut Tokens, session: &mut AstSession) -> Result<Vec<VariantDef>, ()> {
    let mut variants = vec![];

    loop {
        if tokens.is_finished() {
            return Ok(variants);
        }

        let mut attributes = vec![];

        loop {
            if tokens.is_curr_token(TokenKind::Punct(Punct::At)) {
                let at_span = tokens.step().unwrap().span;

                let (deco, _) = parse_decorator(
                    at_span, tokens, session,
                )?;

                attributes.push(Attribute::Decorator(deco));
                continue;
            }

            if tokens.is_curr_token_doc_comment() {
                let curr_span = tokens.peek_span().unwrap();

                attributes.push(Attribute::DocComment(
                    IdentWithSpan::new(
                        tokens.expect_doc_comment().unwrap(),
                        curr_span,
                    )
                ));

                continue;
            }

            break;
        }

        let variant_name = match tokens.expect_ident() {
            Ok(id) => id,
            Err(mut e) => {
                session.push_error(e.set_error_context(
                    ErrorContext::ParsingEnumBody
                ).to_owned());
                return Err(());
            },
        };

        match tokens.step() {
            Some(Token {
                kind: TokenKind::Group { delim, tokens: type_tokens, prefix: b'\0' },
                span: group_span,
            }) => {
                let group_span = *group_span;
                let mut type_tokens = type_tokens.to_vec();
                let mut type_tokens = Tokens::from_vec(&mut type_tokens);
                type_tokens.set_span_end(group_span.last_char());

                match delim {
                    Delim::Paren => {
                        let (args, _) = parse_comma_separated_types(&mut type_tokens, session)?;

                        variants.push(VariantDef {
                            name: variant_name,
                            args: VariantKind::Tuple(args),
                            attributes,
                        });

                        match tokens.consume(TokenKind::comma()) {
                            Err(AstError {
                                kind: AstErrorKind::UnexpectedEnd(_),
                                ..
                            }) => {
                                return Ok(variants);
                            },
                            Err(mut e) => {
                                session.push_error(e.set_error_context(
                                    ErrorContext::ParsingEnumBody
                                ).to_owned());

                                return Err(());
                            },
                            Ok(()) => {},
                        }
                    },
                    Delim::Brace => {
                        let args = parse_struct_body(&mut type_tokens, session, group_span)?;

                        variants.push(VariantDef {
                            name: variant_name,
                            args: VariantKind::Struct(args),
                            attributes,
                        });

                        match tokens.consume(TokenKind::comma()) {
                            Err(AstError {
                                kind: AstErrorKind::UnexpectedEnd(_),
                                ..
                            }) => {
                                return Ok(variants);
                            },
                            Err(mut e) => {
                                session.push_error(e.set_error_context(
                                    ErrorContext::ParsingEnumBody
                                ).to_owned());

                                return Err(());
                            },
                            Ok(()) => {},
                        }
                    },
                    Delim::Bracket => {
                        session.push_error(AstError::unexpected_token(
                            Token::new_group(Delim::Bracket, group_span),
                            ExpectedToken::paren_brace_or_comma(),
                        ).set_error_context(
                            ErrorContext::ParsingEnumBody
                        ).to_owned());
                        return Err(());
                    },
                }
            },
            Some(Token {
                kind: TokenKind::Punct(Punct::Comma),
                ..
            }) => {
                variants.push(VariantDef {
                    name: variant_name,
                    args: VariantKind::Empty,
                    attributes,
                });
            },
            Some(token) => {
                session.push_error(AstError::unexpected_token(
                    token.clone(),
                    ExpectedToken::comma_or_paren(),
                ).set_error_context(
                    ErrorContext::ParsingEnumBody
                ).to_owned());
                return Err(());
            },
            None => {
                variants.push(VariantDef {
                    name: variant_name,
                    args: VariantKind::Empty,
                    attributes,
                });
                return Ok(variants);
            }
        }
    }
}

// generic params are just identifiers separated by commas
// a trailing comma is fine
// for now, empty list is not allowed
fn parse_generic_param_list(tokens: &mut Tokens, session: &mut AstSession) -> Result<Vec<GenericDef>, ()> {
    let lt_span = tokens.peek_span();

    match tokens.peek() {
        Some(Token {
            kind: TokenKind::Punct(Punct::Concat),
            span,
        }) => {
            session.push_error(AstError::empty_generic_list(*span));

            return Err(());
        },
        _ => {},
    }

    if let Err(e) = tokens.consume(TokenKind::lt()) {
        session.push_error(e);
        return Err(());
    }

    let mut params = vec![];

    loop {
        if tokens.is_finished() {
            if params.is_empty() {
                session.push_error(AstError::empty_generic_list(
                    tokens.span_end().unwrap_or(lt_span.unwrap())
                ));

                return Err(());
            }

            else {
                return Ok(params);
            }
        }

        params.push(
            match tokens.expect_ident() {
                Ok(id) => id,
                Err(e @ AstError {
                    kind: AstErrorKind::UnexpectedToken(TokenKind::Punct(Punct::Gt), _),
                    ..
                }) if params.is_empty() => {
                    session.push_error(AstError::empty_generic_list(
                        e.get_first_span().unwrap()
                    ));
                    return Err(());
                },
                Err(e) => {
                    session.push_error(e);
                    return Err(());
                },
            }
        );

        match tokens.step() {
            Some(Token {
                kind: TokenKind::Punct(Punct::Comma),
                ..
            }) => {
                if tokens.is_curr_token(TokenKind::gt()) {
                    tokens.step().unwrap();  // step `>`
                    return Ok(params);
                }

                continue;
            },
            Some(Token {
                kind: TokenKind::Punct(Punct::Gt),
                ..
            }) => {
                return Ok(params);
            },
            Some(token) => {
                session.push_error(AstError::unexpected_token(
                    token.clone(),
                    ExpectedToken::comma_or_gt(),
                ));

                return Err(());
            },
            None => {
                session.push_error(AstError::unexpected_end(
                    tokens.span_end().unwrap_or(SpanRange::dummy()),
                    ExpectedToken::comma_or_gt(),
                ));

                return Err(());
            },
        }
    }
}

// returns None if it's not a struct initialization at all. In that case, it could be a syntax error,
// but the check is done later.
// if it's surely a struct init, it returns Some. Some(Err) is an initialization with a syntax error,
// and Some(Ok) is one without an error
fn try_parse_struct_init(tokens: &mut Tokens, session: &mut AstSession) -> Option<Result<Vec<StructInitDef>, ()>> {
    // `if foo {}` is a syntax error, but the compiler doesn't know the programmer's intent
    // the intent can either be:
    // 1. `foo` is a struct, but the programmer forgot to init its fields
    //    - Sodigy doesn't allow structs without any field, because of this reason.
    //    - If so, the compiler would raise a very awkward error message in this case.
    // 2. `foo` is the condition and the programmer forgot the value of the `if` expression
    //
    // for now, it assumes that the intention is 2
    let mut is_struct_init = false;
    let mut fields = vec![];

    loop {
        if tokens.is_finished() {
            if is_struct_init {
                return Some(Ok(fields));
            }

            else {
                return None;
            }
        }

        let field_name = match tokens.expect_ident() {
            Ok(n) => n,
            Err(mut e) => {
                if is_struct_init {
                    session.push_error(e.set_error_context(
                        ErrorContext::ParsingStructInit
                    ).to_owned());

                    return Some(Err(()));
                }

                else {
                    return None;
                }
            },
        };

        let comma_span = tokens.peek_span();

        if let Err(mut e) = tokens.consume(TokenKind::colon()) {
            if is_struct_init {
                session.push_error(
                    e.set_error_context(
                        ErrorContext::ParsingStructInit
                    ).to_owned()
                );

                return Some(Err(()));
            }

            else {
                return None;
            }
        }

        // Now we're sure that it's a struct initialization,
        // since that it has `IDENT: `.
        else {
            is_struct_init = true;
        }

        let value = match parse_expr(
            tokens,
            session,
            0,
            false,
            Some(ErrorContext::ParsingStructInit),
            comma_span.unwrap(),
        ) {
            Ok(v) => v,
            Err(_) => {
                return Some(Err(()));
            },
        };

        fields.push(StructInitDef {
            field: field_name,
            value,
        });

        match tokens.consume(TokenKind::comma()) {
            Err(AstError {
                kind: AstErrorKind::UnexpectedEnd(_),
                ..
            }) => {
                continue;
            },
            Err(mut e) => {
                session.push_error(
                    e.set_error_context(
                        ErrorContext::ParsingStructInit
                    ).to_owned()
                );
            },
            Ok(_) => {},
        }
    }
}

fn parse_dotted_names(tokens: &mut Tokens, session: &mut AstSession, error_context: Option<ErrorContext>) -> Result<DottedNames, ()> {
    let mut result = vec![];

    match tokens.expect_ident() {
        Ok(id) => {
            result.push(id);
        },
        Err(mut e) => {
            session.push_error(
                e.try_set_error_context(
                    error_context
                ).to_owned()
            );
            return Err(());
        },
    }

    while tokens.is_curr_token(TokenKind::dot()) {
        tokens.step().unwrap();

        match tokens.expect_ident() {
            Ok(id) => {
                result.push(id);
            },
            Err(mut e) => {
                session.push_error(
                    e.try_set_error_context(
                        error_context
                    ).to_owned()
                );
                return Err(());
            },
        }
    }

    Ok(result)
}

// "let" NAME ("<" GENERICS ">")? ("(" ARGS ")")? (":" TYPE)? "=" EXPR ";"
// "let" "pattern" PATTERN "=" EXPR ";"
// "let" "enum" NAME ("<" GENERICS ">")? "=" ENUM_BODY ";"
// "let" "struct" NAME ("<" GENERICS ">")? "=" STRUCT_BODY ";"
// The keyword `let` is already consumed
fn parse_let_statement(
    tokens: &mut Tokens,
    session: &mut AstSession,
    allows_generics: bool,
    attributes: Vec<Attribute>,
) -> Result<Let, ()> {
    let result = match tokens.step() {
        Some(Token {
            kind: TokenKind::Keyword(k),
            span,
        }) => {
            match *k {
                Keyword::Pattern => {
                    let pattern = parse_pattern_full(tokens, session)?;
                    let assign_span = tokens.peek_span();

                    if let Err(mut e) = tokens.consume(TokenKind::assign()) {
                        e.set_error_context(ErrorContext::ParsingLetStatement);

                        if let Some(Token {
                            kind: TokenKind::Punct(Punct::At),
                            ..
                        }) = tokens.peek() {
                            e.set_message(String::from("To bind a name to a pattern, the name must come before the pattern, not after it."));
                        }

                        session.push_error(e);
                        return Err(());
                    }

                    let assign_span = assign_span.unwrap();

                    let expr = parse_expr(tokens, session, 0, false, Some(ErrorContext::ParsingLetStatement), assign_span)?;

                    Let::pattern(pattern, expr, attributes)
                },
                k @ (Keyword::Enum | Keyword::Struct) => {
                    let name = match tokens.expect_ident() {
                        Ok(id) => id,
                        Err(mut e) => {
                            session.push_error(
                                e.set_error_context(
                                    ErrorContext::ParsingLetStatement,
                                ).to_owned()
                            );
                            return Err(());
                        },
                    };

                    let generics = if tokens.is_curr_token(TokenKind::lt()) || tokens.is_curr_token(TokenKind::Punct(Punct::Concat)) {
                        parse_generic_param_list(tokens, session)?
                    } else {
                        vec![]
                    };

                    if !generics.is_empty() && !allows_generics {
                        session.push_error(AstError::no_generics_allowed(
                            tokens.get_previous_generic_span().unwrap()
                        ).set_error_context(
                            ErrorContext::ParsingLetStatement
                        ).to_owned());
                    }

                    if let Err(mut e) = tokens.consume(TokenKind::assign()) {
                        session.push_error(
                            e.set_error_context(
                                ErrorContext::ParsingLetStatement,
                            ).to_owned()
                        );
                        return Err(());
                    }

                    let group_span = tokens.peek_span();

                    match tokens.expect_group(Delim::Brace) {
                        Ok(group_tokens) => {
                            let group_span = group_span.unwrap();
                            let mut group_tokens = group_tokens.to_vec();
                            let mut group_tokens = Tokens::from_vec(&mut group_tokens);
                            let last_token_span = group_tokens.span_end().unwrap_or(group_span);

                            if let Keyword::Enum = k {
                                Let::enum_(
                                    name,
                                    generics,
                                    parse_enum_body(&mut group_tokens, session)?,
                                    attributes,
                                )
                            }

                            else {  // struct
                                Let::struct_(
                                    name,
                                    generics,
                                    parse_struct_body(&mut group_tokens, session, last_token_span)?,
                                    attributes,
                                )
                            }
                        },
                        Err(mut e) => {
                            session.push_error(
                                e.set_error_context(
                                    ErrorContext::ParsingLetStatement,
                                ).to_owned()
                            );
                            return Err(());
                        },
                    }
                },
                k => {
                    session.push_error(AstError::unexpected_token(
                        Token {
                            kind: TokenKind::Keyword(k),
                            span: *span,
                        },
                        ExpectedToken::let_statement(),
                    ));

                    return Err(());
                },
            }
        },
        Some(Token {
            kind: TokenKind::Identifier(id),
            span,
        }) => {
            let name = IdentWithSpan::new(*id, *span);
            let mut has_error = false;

            let generics = if tokens.is_curr_token(TokenKind::lt()) || tokens.is_curr_token(TokenKind::Punct(Punct::Concat)) {
                parse_generic_param_list(tokens, session)?
            } else {
                vec![]
            };

            if !generics.is_empty() && !allows_generics {
                session.push_error(AstError::no_generics_allowed(
                    tokens.get_previous_generic_span().unwrap()
                ).set_error_context(
                    ErrorContext::ParsingLetStatement
                ).to_owned());
            }

            let args = match tokens.peek() {
                Some(Token {
                    kind: TokenKind::Group {
                        delim: Delim::Paren,
                        tokens: args_tokens,
                        prefix: b'\0',
                    },
                    span: arg_span,
                }) => {
                    let arg_span = *arg_span;
                    let mut args_tokens = args_tokens.to_vec();
                    let mut args_tokens = Tokens::from_vec(&mut args_tokens);
                    args_tokens.set_span_end(arg_span.last_char());

                    tokens.step().unwrap();

                    match parse_arg_defs(&mut args_tokens, session) {
                        Ok(a) => Some(a),
                        _ => {
                            has_error = true;
                            None
                        },
                    }
                },
                _ => None,
            };

            let return_ty = if tokens.is_curr_token(TokenKind::colon()) {
                let colon_span = tokens.step().unwrap().span;

                Some(parse_type_def(tokens, session, colon_span)?)
            } else {
                None
            };

            let assign_span = tokens.peek_span();

            if let Err(mut e) = tokens.consume(TokenKind::assign()) {
                if return_ty.is_none() {
                    e.add_expected_token(TokenKind::colon()).unwrap();

                    // I want to tell the users who use "->" to annotate a return type of a function
                    if tokens.match_first_tokens(&vec![
                        TokenKind::sub(),
                        TokenKind::gt(),
                    ]) {
                        e.set_message(String::from("Sodigy does not use `->` to annotate a return type of a function. Try `:` instead."));
                    }

                    if args.is_none() {
                        e.add_expected_token(TokenKind::new_group(Delim::Paren)).unwrap();

                        if generics.is_empty() {
                            e.add_expected_token(TokenKind::lt()).unwrap();
                        }
                    }

                    e.set_error_context(ErrorContext::ParsingLetStatement);
                }

                // it's impossible to know the user's intention here.
                // it chose ParsingTypeAnnotation because that context has
                // more error messages to provide
                else {
                    e.set_error_context(ErrorContext::ParsingTypeAnnotation);
                }

                session.push_error(e);
                return Err(());
            }

            let return_val = parse_expr(
                tokens,
                session,
                0,
                false,
                Some(ErrorContext::ParsingFuncBody),
                assign_span.unwrap(),
            )?;

            if has_error {
                return Err(());
            }

            else {
                Let::def(name, generics, args, return_ty, return_val, attributes)
            }
        },
        Some(token) => {
            let mut e = AstError::unexpected_token(
                token.clone(),
                ExpectedToken::let_statement(),
            ).set_error_context(
                ErrorContext::ParsingLetStatement
            ).to_owned();

            if token.is_group(Delim::Paren)
            || token.is_group(Delim::Bracket)
            || matches!(token.kind, TokenKind::Punct(Punct::Dollar)) {
                e.set_message("If you meant to destruct a pattern, use `let pattern` instead of `let`.".to_string());
            }

            session.push_error(e);

            return Err(());
        },
        None => {
            session.push_error(AstError::unexpected_end(
                tokens.span_end().unwrap_or(SpanRange::dummy()),
                ExpectedToken::let_statement(),
            ).set_error_context(
                ErrorContext::ParsingLetStatement
            ).to_owned());

            return Err(());
        },
    };

    if let Err(mut e) = tokens.consume(TokenKind::semi_colon()) {
        e.set_error_context(ErrorContext::ParsingLetStatement);

        if tokens.is_curr_token(TokenKind::Keyword(Keyword::Let)) {
            e.set_message("Use `;` before the keyword `let` to separate statements.".to_string());
        }

        session.push_error(e);
        return Err(());
    }

    Ok(result)
}
