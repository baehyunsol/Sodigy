use crate::{
    ArgDef,
    BranchArm,
    err::{ExpectedToken, AstError, AstErrorKind},
    expr::{Expr, ExprKind},
    format_string_into_expr,
    GenericDef,
    IdentWithSpan,
    LocalDef,
    MatchArm,
    ops::{
        call_binding_power,
        index_binding_power,
        infix_binding_power,
        path_binding_power,
        postfix_binding_power,
        prefix_binding_power,
        InfixOp,
        PostfixOp,
        PrefixOp,
    },
    Pattern,
    ScopeDef,
    session::AstSession,
    stmt::{
        Attribute,
        Decorator,
        EnumDef,
        FieldDef,
        FuncDef,
        Stmt,
        StmtKind,
        StructDef,
        Use,
        VariantDef,
        VariantKind,
    },
    tokens::Tokens, Token, TokenKind,
    TypeDef,
    value::ValueKind,
};
use sodigy_err::{ErrorContext, SodigyError};
use sodigy_intern::InternedString;
use sodigy_keyword::Keyword;
use sodigy_lex::QuoteKind;
use sodigy_parse::{Delim, Punct};
use sodigy_span::SpanRange;

pub fn parse_stmts(tokens: &mut Tokens, session: &mut AstSession) -> Result<(), ()> {
    loop {
        match tokens.step() {
            Some(Token {
                kind: TokenKind::Keyword(k),
                span: keyword_span,
            }) => {
                let keyword = *k;
                let keyword_span = *keyword_span;

                match keyword {
                    Keyword::Def => {
                        // 'def' IDENTIFIER ('<' GENERICS '>')? ('(' ARGS ')')? (':' TYPE)? '=' EXPR ';'
                        let mut span = keyword_span;

                        let def_name = match tokens.expect_ident() {
                            Ok(id) => id,
                            Err(mut e) => {
                                session.push_error(
                                    e.set_err_context(
                                        ErrorContext::ParsingFuncName,
                                    ).to_owned()
                                );
                                tokens.march_until_stmt();
                                continue;
                            },
                        };

                        let generics = if tokens.is_curr_token(TokenKind::Punct(Punct::Lt)) {
                            match parse_generic_param_list(tokens, session) {
                                Ok(g) => g,
                                Err(()) => {
                                    tokens.march_until_stmt();
                                    continue;
                                }
                            }
                        } else {
                            vec![]
                        };

                        let args = match tokens.peek() {
                            Some(Token {
                                kind: TokenKind::Group {
                                    delim: Delim::Paren,
                                    tokens: args_tokens,
                                    prefix: b'\0',
                                },
                                ..
                            }) => {
                                let mut args_tokens = args_tokens.to_vec();
                                let mut args_tokens = Tokens::from_vec(&mut args_tokens);

                                tokens.step().unwrap();

                                let arg_defs = parse_arg_defs(&mut args_tokens, session)?;

                                for arg in arg_defs.iter() {
                                    if !arg.has_type() {
                                        session.push_error(
                                            AstError::func_arg_without_type(
                                                *def_name.id(),
                                                arg.name,
                                            )
                                        );

                                        // this error doesn't block parsing
                                    }
                                }

                                Some(arg_defs)
                            },
                            _ => None,
                        };

                        let ret_type = if tokens.is_curr_token(TokenKind::Punct(Punct::Colon)) {
                            let colon_span = tokens.peek_span().unwrap();
                            tokens.step().unwrap();

                            Some(parse_type_def(tokens, session, Some(ErrorContext::ParsingFuncRetType), colon_span)?)
                        } else {
                            None
                        };

                        let assign_span = tokens.peek_span();

                        if let Err(mut e) = tokens.consume(TokenKind::Punct(Punct::Assign)) {
                            session.push_error(
                                e.set_err_context(
                                    ErrorContext::ParsingFuncBody,
                                ).to_owned()
                            );
                            tokens.march_until_stmt();
                            continue;
                        }

                        let ret_val = match parse_expr(
                            tokens,
                            session,
                            0,
                            false,
                            Some(ErrorContext::ParsingFuncBody),
                            assign_span.unwrap(),
                        ) {
                            Ok(v) => v,
                            Err(()) => {
                                tokens.march_until_stmt();
                                continue;
                            },
                        };

                        let semi_colon_span = tokens.peek_span();

                        if let Err(mut e) = tokens.consume(TokenKind::Punct(Punct::SemiColon)) {
                            session.push_error(
                                e.set_err_context(
                                    ErrorContext::ParsingFuncBody,
                                ).to_owned()
                            );
                            tokens.march_until_stmt();
                            continue;
                        }

                        let semi_colon_span = semi_colon_span.unwrap();
                        span = span.merge(semi_colon_span);

                        session.push_stmt(Stmt {
                            kind: StmtKind::Func(FuncDef {
                                name: def_name,
                                generics,
                                args,
                                ret_type,
                                ret_val,
                            }),
                            span,
                        });
                    },
                    def_type @ (Keyword::Enum | Keyword::Struct) => {
                        // ('enum' | 'struct') IDENTIFIER ('<' GENERICS '>')? '{' ENUM_BODY | STRUCT_BODY '}'
                        // VARIANT: IDENTIFIER ('(' TYPES ')')?

                        let mut span = keyword_span;
                        let def_name = match tokens.expect_ident() {
                            Ok(id) => id,
                            Err(e) => {
                                session.push_error(e);
                                tokens.march_until_stmt();
                                continue;
                            },
                        };

                        let generics = if tokens.is_curr_token(TokenKind::Punct(Punct::Lt)) {
                            parse_generic_param_list(tokens, session)?
                        } else {
                            vec![]
                        };

                        let last_span = tokens.peek_span();

                        match tokens.expect_group(Delim::Brace) {
                            Ok(mut body_tokens) => {
                                let mut tokens = Tokens::from_vec(&mut body_tokens);
                                let last_span = last_span.unwrap();

                                if let Keyword::Enum = def_type {
                                    match parse_enum_body(&mut tokens, session) {
                                        Ok(variants) => {
                                            span = span.merge(last_span);

                                            session.push_stmt(Stmt {
                                                kind: StmtKind::Enum(EnumDef {
                                                    name: def_name,
                                                    generics,
                                                    variants,
                                                }),
                                                span,
                                            });
                                        },
                                        Err(_) => {
                                            tokens.march_until_stmt();
                                            continue;
                                        }
                                    }
                                }

                                // struct
                                else {
                                    match parse_struct_body(&mut tokens, session) {
                                        Ok(fields) => {
                                            span = span.merge(last_span);

                                            session.push_stmt(Stmt {
                                                kind: StmtKind::Struct(StructDef {
                                                    name: def_name,
                                                    generics,
                                                    fields,
                                                }),
                                                span,
                                            });
                                        },
                                        Err(_) => {
                                            tokens.march_until_stmt();
                                            continue;
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                session.push_error(e);
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

                        if let Err(e) = tokens.consume(TokenKind::Punct(Punct::SemiColon)) {
                            session.push_error(e);
                            tokens.march_until_stmt();
                            continue;
                        }

                        session.push_stmt(Stmt {
                            kind: StmtKind::Module(mod_name),
                            span,
                        });
                    },
                    Keyword::Use => {
                        match parse_use(tokens, session, keyword_span) {
                            Ok(u) => {
                                session.push_stmt(Stmt {
                                    kind: StmtKind::Use(u),
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
                        session.push_error(AstError::unexpected_token(
                            Token::new_keyword(unexpected_keyword, keyword_span),
                            ExpectedToken::stmt(),
                        ));
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
                        session.push_stmt(Stmt {
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

                session.push_stmt(Stmt {
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

    session.err_if_has_err()
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

        match tokens.consume(TokenKind::Punct(Punct::Comma)) {
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
                    session.push_error(AstError::unexpected_token(
                        Token::new_punct(punct, prefix_op_span),
                        ExpectedToken::expr(),
                    ).set_message(
                        format!("`{punct}` is not a valid prefix operator.")
                    ).try_set_err_context(
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

            loop {
                branch_arms.push(parse_branch_arm(tokens, session, span)?);

                if !tokens.is_curr_token(TokenKind::Keyword(Keyword::Else)) {
                    break;
                }

                // step `else`
                tokens.step().unwrap();
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

                    Expr {
                        kind: ExprKind::Match {
                            value: Box::new(value),
                            arms: parse_match_body(&mut match_body_tokens, session, group_span)?,
                        },
                        span: span.merge(last_token_span),
                    }
                },
                Err(mut e) => {
                    session.push_error(
                        e.set_err_context(
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
                ).try_set_err_context(
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
                    let (args, value) = parse_lambda_body(&mut tokens, session, span)?;

                    Expr {
                        kind: ExprKind::Value(ValueKind::Lambda { args, value: Box::new(value) }),
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
                    ).try_set_err_context(
                        error_context,
                    ).to_owned());
                    return Err(());
                }
            }

            else {
                match delim {
                    Delim::Paren => {
                        let mut tokens = Tokens::from_vec(&mut tokens);

                        match parse_comma_separated_exprs(&mut tokens, session) {
                            Ok((elems, has_trailing_comma)) if !has_trailing_comma && elems.len() == 1 => {
                                // TODO: do I have to record that it's inside parenthesis?
                                elems[0].clone()
                            },
                            Ok((elems, _)) => {
                                Expr {
                                    kind: ExprKind::Value(ValueKind::Tuple(elems)),
                                    span,
                                }
                            },
                            Err(()) => {
                                return Err(());
                            },
                        }
                    },
                    Delim::Bracket => {
                        let mut tokens = Tokens::from_vec(&mut tokens);
                        let (elems, _) = parse_comma_separated_exprs(&mut tokens, session)?;

                        Expr {
                            kind: ExprKind::Value(ValueKind::List(elems)),
                            span,
                        }
                    },
                    Delim::Brace => {
                        let mut tokens = Tokens::from_vec(&mut tokens);

                        Expr {
                            kind: ExprKind::Value(ValueKind::Scope(parse_scope_block(&mut tokens, session, span)?)),
                            span,
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
                        s: *content,
                        is_binary,
                    }),
                    span,
                },
                QuoteKind::Single => if is_binary {
                    // There are no binary chars, because `Char`s in Sodigy are just integers
                    session.push_error(AstError::binary_char(span).try_set_err_context(
                        error_context,
                    ).to_owned());
                    return Err(());
                } else {
                    let mut chars = session.unintern_string(*content).unwrap().iter();
                    let first_c = match chars.next() {
                        Some(c) => c,
                        None => {
                            session.push_error(AstError::empty_char_literal(span).try_set_err_context(
                                error_context,
                            ).to_owned());
                            return Err(());
                        },
                    };

                    if let Some(_) = chars.next() {
                        session.push_error(AstError::too_long_char_literal(span).try_set_err_context(
                            error_context,
                        ).to_owned());
                        return Err(());
                    }

                    Expr {
                        kind: ExprKind::Value(ValueKind::Char(*first_c as char)),
                        span,
                    }
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
                    Token::new_doc_comment(InternedString::dotdotdot(), *span),
                    ExpectedToken::expr(),
                ).try_set_err_context(
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

            session.err_if_has_err()?;

            Expr {
                kind: ExprKind::Value(ValueKind::Format(
                    elems
                )),
                span,
            }
        },
        None => {
            if do_nothing_when_failed {
                return Err(());
            }

            else {
                session.push_error(AstError::unexpected_end(
                    tokens.span_end().unwrap_or(parent_span.last_char()),
                    ExpectedToken::expr(),
                ).try_set_err_context(
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
                    // `..` can both be inclusive and exclusive!
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
                            session.push_error(e.try_set_err_context(
                                error_context,
                            ).to_owned());
                            return Err(());
                        },
                    };

                    lhs = Expr {
                        kind: ExprKind::Path { pre: Box::new(lhs), post: rhs },
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

                        let rhs = parse_expr(tokens, session, r_bp, false, error_context, punct_span)?;

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
                    ).try_set_err_context(
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
                            let rhs = parse_expr(&mut index_tokens, session, 0, false, error_context, span)?;

                            if !index_tokens.is_finished() {
                                session.push_error(AstError::unexpected_token(
                                    index_tokens.peek().unwrap().clone(),
                                    ExpectedToken::nothing(),
                                ).try_set_err_context(
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
                            let (args, _) = parse_comma_separated_exprs(&mut index_tokens, session)?;

                            lhs = Expr {
                                kind: ExprKind::Call {
                                    functor: Box::new(lhs),
                                    args,
                                },
                                span,
                            };
                            continue;
                        },
                        Delim::Brace => {
                            tokens.backward().unwrap();
                            break;
                        },
                    }
                }
            },
            _ => {
                break;
            }
        }
    }

    Ok(lhs)
}

pub fn parse_type_def(
    tokens: &mut Tokens,
    session: &mut AstSession,
    error_context: Option<ErrorContext>,

    // when `tokens` is empty, it uses this span for the error message
    parent_span: SpanRange,
) -> Result<TypeDef, ()> {
    Ok(TypeDef::from_expr(parse_expr(
        tokens,
        session,
        0,
        false,
        error_context,
        parent_span,
    )?))
}

// this function allows a trailing comma and args without type annotations
// it's your responsibility to check type annotations
fn parse_arg_defs(tokens: &mut Tokens, session: &mut AstSession) -> Result<Vec<ArgDef>, ()> {
    let mut args = vec![];

    loop {
        if tokens.is_finished() {
            return Ok(args);
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
                    Some(ErrorContext::ParsingFuncArgs),
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
                });

                continue;
            },
            Some(token) => {
                session.push_error(AstError::unexpected_token(
                    token.clone(),
                    ExpectedToken::comma_or_colon(),
                ));

                return Err(());
            },
            None => {
                args.push(ArgDef {
                    name: arg_name,
                    ty: arg_type,
                    has_question_mark,
                });
                return Ok(args);
            },
        }

        match tokens.consume(TokenKind::Punct(Punct::Comma)) {
            Ok(()) => {},
            Err(AstError {
                kind: AstErrorKind::UnexpectedEnd(_),
                ..
            }) => {
                args.push(ArgDef {
                    name: arg_name,
                    ty: arg_type,
                    has_question_mark,
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
) -> Result<ScopeDef, ()> {
    if tokens.is_finished() {
        session.push_error(AstError::empty_scope_block(span));
        return Err(());
    }

    let mut defs = vec![];

    loop {
        if !tokens.is_curr_token(TokenKind::Keyword(Keyword::Let)) {
            break;
        }

        // step `let`
        let let_span = tokens.peek_span().unwrap();
        tokens.step().unwrap();

        let pattern = parse_pattern(tokens, session, true)?;
        let assign_span = tokens.peek_span();

        if let Err(e) = tokens.consume(TokenKind::Punct(Punct::Assign)) {
            session.push_error(e);
            return Err(());
        }

        let value = parse_expr(
            tokens,
            session,
            0,
            false,
            Some(ErrorContext::ParsingScopeBlock),
            assign_span.unwrap(),
        )?;

        if let Err(e) = tokens.consume(TokenKind::Punct(Punct::SemiColon)) {
            session.push_error(e);
            return Err(());
        }

        defs.push(LocalDef {
            let_span,
            pattern,
            value,
        });
    }

    let value = parse_expr(
        tokens,
        session,
        0,
        false,
        Some(ErrorContext::ParsingScopeBlock),
        span,
    )?;

    Ok(ScopeDef { defs, value: Box::new(value) })
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

                // TODO: `?` after arg
                // It's tough -> `x?` can both be an arg and an expr

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
                            Some(ErrorContext::ParsingLambdaBody),
                            colon_span,
                        )?;

                        args.push(ArgDef {
                            name: curr_arg,
                            ty: Some(ty_anno),
                            has_question_mark: false,  // TODO
                        });

                        if let Err(e) = tokens.consume(TokenKind::Punct(Punct::Comma)) {
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
                            has_question_mark: false,  // TODO
                        });
                        continue;
                    },
                    Some(_) => {
                        /* the last ident is an expr */
                        tokens.backward().unwrap();
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
                    AstError::empty_match_body(span).set_err_context(
                        ErrorContext::ParsingMatchBody
                    ).to_owned()
                );
                return Err(());
            }

            else {
                return Ok(arms);
            }
        }

        let pattern = parse_pattern(tokens, session, false)?;
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

                if let Err(mut e) = tokens.consume(TokenKind::Punct(Punct::RArrow)) {
                    session.push_error(e.set_err_context(
                        ErrorContext::ParsingMatchBody
                    ).to_owned());
                    return Err(());
                }
            },
            Some(token) => {
                session.push_error(AstError::unexpected_token(
                    token.clone(),
                    ExpectedToken::guard_or_arrow(),
                ).set_err_context(
                    ErrorContext::ParsingMatchBody
                ).to_owned());
                return Err(());
            },
            None => {
                session.push_error(AstError::unexpected_end(
                    span,
                    ExpectedToken::guard_or_arrow(),
                ).set_err_context(
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
        });

        match tokens.consume(TokenKind::Punct(Punct::Comma)) {
            Err(AstError {
                kind: AstErrorKind::UnexpectedEnd(_),
                ..
            }) => {
                return Ok(arms);
            },
            Err(mut e) => {
                session.push_error(e.set_err_context(
                    ErrorContext::ParsingMatchBody
                ).to_owned());
                return Err(());
            },
            _ => {},
        }
    }
}

type LetMode = bool;

// `let_mode` tells the function how to deal with a single-ident pattern
// in `let a = foo();`, `a` is a newly-bound name.
// in `match foo() { a => 3 }`, `a` is a name of an already-defined variant.

// TODO: type annotations in patterns e.g. let ($x: Int, $y: Int) = foo();
// -> any pattern may have a type annotation
// -> I need full spec for patterns
fn parse_pattern(
    tokens: &mut Tokens,
    session: &mut AstSession,
    let_mode: LetMode,
) -> Result<Pattern, ()> {
    // TODO
    session.push_error(AstError::todo("pattern", tokens.peek_span().unwrap().clone()));
    Err(())
}

// it expects either `if` or `{ ... }`
fn parse_branch_arm(
    tokens: &mut Tokens,
    session: &mut AstSession,

    // when `tokens` is empty, it uses this span for the error message
    parent_span: SpanRange,
) -> Result<BranchArm, ()> {
    match tokens.step() {
        Some(Token {
            kind: TokenKind::Keyword(Keyword::If),
            span: if_span,
        }) => {
            let if_span = *if_span;

            if tokens.is_curr_token(TokenKind::Keyword(Keyword::Let)) {
                /* if-let statement */
                todo!()
            }

            let cond = parse_expr(tokens, session, 0, false, None, if_span)?;
            let span = tokens.peek_span();

            match tokens.expect_group(Delim::Brace) {
                Ok(tokens) => {
                    let mut val_tokens = tokens.to_vec();
                    let mut val_tokens = Tokens::from_vec(&mut val_tokens);
                    let span = span.unwrap();

                    let value = parse_scope_block(&mut val_tokens, session, span)?;
                    let value = Expr {
                        kind: ExprKind::Value(ValueKind::Scope(value)),
                        span,
                    };

                    Ok(BranchArm {
                        cond: Some(cond),
                        value,
                    })
                },
                Err(e) => {
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

            let value = parse_scope_block(&mut val_tokens, session, span)?;
            let value = Expr {
                kind: ExprKind::Value(ValueKind::Scope(value)),
                span,
            };

            Ok(BranchArm {
                cond: None,
                value,
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

    while let Ok(()) = tokens.consume(TokenKind::Punct(Punct::Dot)) {
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

// use a;
// use a, b, c;
// use {a, b, c};
// use a.b;
// use a.{b, c, d};
// use a.{b, c, d}, e, f.{g, h as i};
// use a as b;
// use a.b as c;
// use a.{b as c, d, e};
fn parse_use(tokens: &mut Tokens, session: &mut AstSession, span: SpanRange) -> Result<Use, ()> {
    match tokens.step() {
        Some(Token {
            kind: TokenKind::Identifier(id),
            span: id_span,
        }) => {
            session.push_error(AstError::todo("use", span));
            return Err(());
        },
        Some(Token {
            kind: TokenKind::Group {
                delim: Delim::Brace,
                tokens: inner_tokens,
                prefix: b'\0',
            },
            span: group_span,
        }) => {
            let group_span = *group_span;
            let mut inner_tokens = inner_tokens.to_vec();
            let mut inner_tokens = Tokens::from_vec(&mut inner_tokens);

            parse_use(&mut inner_tokens, session, group_span)
        },
        Some(token) => {
            session.push_error(AstError::unexpected_token(
                token.clone(),
                ExpectedToken::ident_or_brace(),
            ));
            return Err(());
        },
        None => {
            session.push_error(AstError::unexpected_end(
                span,
                ExpectedToken::ident_or_brace(),
            ));
            return Err(());
        },
    }
}

// tokens inside `{ ... }`
fn parse_struct_body(tokens: &mut Tokens, session: &mut AstSession) -> Result<Vec<FieldDef>, ()> {
    let mut fields = vec![];

    loop {
        if tokens.is_finished() {
            return Ok(fields);
        }

        let mut attributes = vec![];

        loop {
            if tokens.is_curr_token(TokenKind::Punct(Punct::At)) {
                let at_span = tokens.peek_span().unwrap();
                tokens.step().unwrap();

                let (deco, _) = parse_decorator(
                    at_span, tokens, session,
                )?;

                attributes.push(Attribute::Decorator(deco));
                continue;
            }

            if tokens.is_curr_token_doc_comment() {
                attributes.push(Attribute::DocComment(
                    tokens.expect_doc_comment().unwrap_or_else(|_| unreachable!())
                ));
                continue;
            }

            break;
        }

        let field_name = match tokens.expect_ident() {
            Ok(id) => id,
            Err(mut e) => {
                session.push_error(e.set_err_context(
                    ErrorContext::ParsingStructBody
                ).to_owned());
                return Err(());
            },
        };

        let colon_span = tokens.peek_span();

        if let Err(mut e) = tokens.consume(TokenKind::Punct(Punct::Colon)) {
            session.push_error(e.set_err_context(
                ErrorContext::ParsingStructBody
            ).to_owned());
            return Err(());
        }

        let field_ty = parse_type_def(
            tokens,
            session,
            Some(ErrorContext::ParsingStructBody),
            colon_span.unwrap(),
        )?;

        fields.push(FieldDef {
            name: field_name,
            ty: field_ty,
            attributes,
        });

        match tokens.consume(TokenKind::Punct(Punct::Comma)) {
            Err(AstError {
                kind: AstErrorKind::UnexpectedEnd(_),
                ..
            }) => {
                return Ok(fields);
            },
            Err(mut e) => {
                session.push_error(e.set_err_context(
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
                let at_span = tokens.peek_span().unwrap();
                tokens.step().unwrap();

                let (deco, _) = parse_decorator(
                    at_span, tokens, session,
                )?;

                attributes.push(Attribute::Decorator(deco));
                continue;
            }

            if tokens.is_curr_token_doc_comment() {
                attributes.push(Attribute::DocComment(
                    tokens.expect_doc_comment().unwrap_or_else(|_| unreachable!())
                ));
                continue;
            }

            break;
        }

        let variant_name = match tokens.expect_ident() {
            Ok(id) => id,
            Err(mut e) => {
                session.push_error(e.set_err_context(
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

                match delim {
                    Delim::Paren => {
                        let (args, _) = parse_comma_separated_types(&mut type_tokens, session)?;

                        variants.push(VariantDef {
                            name: variant_name,
                            args: VariantKind::Tuple(args),
                            attributes,
                        });

                        match tokens.consume(TokenKind::Punct(Punct::Comma)) {
                            Err(AstError {
                                kind: AstErrorKind::UnexpectedEnd(_),
                                ..
                            }) => {
                                return Ok(variants);
                            },
                            Err(mut e) => {
                                session.push_error(e.set_err_context(
                                    ErrorContext::ParsingEnumBody
                                ).to_owned());

                                return Err(());
                            },
                            Ok(()) => {},
                        }
                    },
                    Delim::Brace => {
                        let args = parse_struct_body(&mut type_tokens, session)?;

                        variants.push(VariantDef {
                            name: variant_name,
                            args: VariantKind::Struct(args),
                            attributes,
                        });

                        match tokens.consume(TokenKind::Punct(Punct::Comma)) {
                            Err(AstError {
                                kind: AstErrorKind::UnexpectedEnd(_),
                                ..
                            }) => {
                                return Ok(variants);
                            },
                            Err(mut e) => {
                                session.push_error(e.set_err_context(
                                    ErrorContext::ParsingEnumBody
                                ).to_owned());

                                return Err(());
                            },
                            Ok(()) => {},
                        }
                    },
                    // TODO: there must be something cool I can do with square brackets
                    Delim::Bracket => {
                        session.push_error(AstError::unexpected_token(
                            Token::new_group(Delim::Bracket, group_span),
                            ExpectedToken::paren_brace_or_comma(),
                        ).set_err_context(
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
                ).set_err_context(
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
// empty list is not allowed
fn parse_generic_param_list(tokens: &mut Tokens, session: &mut AstSession) -> Result<Vec<GenericDef>, ()> {
    let lt_span = tokens.peek_span();

    if let Err(e) = tokens.consume(TokenKind::Punct(Punct::Lt)) {
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
