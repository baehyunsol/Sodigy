use super::{Decorator, FuncDef, ModDef, Stmt, Use, use_case_to_tokens};
use crate::err::{ExpectedToken, ParamType, ParseError};
use crate::expr::parse_expr;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::token::{Delimiter, Keyword, OpToken, Token, TokenKind, TokenList};
use std::collections::HashSet;

/// If it returns `Err(())`, the actual errors are in `session`.
pub fn parse_stmts(tokens: &mut TokenList, session: &mut LocalParseSession) -> Result<Vec<Stmt>, ()> {
    let mut result = vec![];

    while !tokens.is_eof() {
        match parse_stmt(tokens) {
            Ok(s) => { result.push(s); },
            Err(e) => {
                session.add_error(e);

                // there' already an error, but it'd helpful for the programmer if the compiler can find more errors
                tokens.march_until_stmt_begin();
            }
        }
    }

    session.err_if_has_error()?;
    Ok(result)
}

pub fn parse_stmt(tokens: &mut TokenList) -> Result<Stmt, ParseError> {
    assert!(!tokens.is_eof(), "Internal Compiler Error 9C5927A4A12");

    let curr_span = tokens
        .peek_curr_span()
        .expect("Internal Compiler Error 6A3B6B7A132");

    if tokens.consume(TokenKind::keyword_use()) {
        // one `use` may generate multiple `Stmt`s, but the return type doesn't allow that
        // so it may modify `tokens` to add `use` cases it found
        match parse_use(tokens, curr_span, true) {
            Ok(mut cases) => {
                assert!(cases.len() > 0, "Internal Compiler Error 60A138A38C9");

                while cases.len() > 1 {
                    tokens.append(use_case_to_tokens(
                        cases.pop().expect("Internal Compiler Error ABD09371688"),
                    ));
                }

                Ok(Stmt::Use(cases[0].clone()))
            }
            Err(e) => {
                return Err(e.set_span_of_eof(curr_span));
            }
        }

    } else if tokens.consume(TokenKind::Keyword(Keyword::Module)) {
        let module_name = match tokens.step_identifier_strict() {
            Ok(id) => id,
            Err(e) => {
                return Err(e.set_span_of_eof(curr_span));
            }
        };

        tokens
            .consume_token_or_error(vec![TokenKind::semi_colon()])
            .map_err(|e| e.set_span_of_eof(curr_span))?;

        Ok(Stmt::Module(ModDef::new(module_name, curr_span)))
    } else if tokens.consume(TokenKind::Operator(OpToken::At)) {
        let mut names = vec![];

        let name = match tokens.step_identifier_strict() {
            Ok(id) => id,
            Err(e) => {
                return Err(e.set_span_of_eof(curr_span));
            }
        };

        names.push(name);

        while tokens.consume(TokenKind::dot()) {
            names.push(match tokens.step_identifier_strict() {
                Ok(id) => id,
                Err(e) => {
                    return Err(e.set_span_of_eof(curr_span));
                }
            });
        }

        let (args, no_args) = match tokens.step_func_args() {
            Some(Ok(args)) => (args, false),
            Some(Err(e)) => {
                return Err(e.set_span_of_eof(curr_span));
            }
            None => (vec![], true)
        };

        Ok(Stmt::Decorator(Decorator {
            names,
            args,
            no_args,
            span: curr_span,
        }))
    } else if tokens.consume(TokenKind::keyword_def()) {
        let name = match tokens.step_identifier_strict() {
            Ok(id) => id,
            Err(e) => {
                return Err(e.set_span_of_eof(curr_span));
            }
        };

        let generics = match tokens.step_generic_defs() {
            Some(generics) => generics?,
            None => vec![],
        };

        let (args, is_const) = match tokens.step_func_def_args() {
            Some(Ok(args)) => (args, false),
            Some(Err(e)) => {
                return Err(e);
            }
            None => (vec![], true),
        };

        let mut arg_names = HashSet::with_capacity(args.len());
        let mut generic_names = HashSet::with_capacity(generics.len());

        for arg in args.iter() {
            if !arg_names.insert(arg.name) {
                return Err(ParseError::multi_def(arg.name, arg.span, ParamType::FuncParam));
            }

            if arg.ty.is_none() {
                return Err(ParseError::untyped_arg(arg.name, name, arg.span));
            }
        }

        for generic in generics.iter() {
            if !generic_names.insert(generic.name) {
                return Err(ParseError::multi_def(generic.name, generic.span, ParamType::FuncGeneric));
            }

            if arg_names.contains(&generic.name) {
                return Err(ParseError::multi_def(generic.name, generic.span, ParamType::FuncGenericAndParam));
            }
        }

        tokens
            .consume_token_or_error(vec![TokenKind::colon()])
            .map_err(|e| e.set_span_of_eof(curr_span))?;

        let ret_type = match tokens.step_type() {
            Some(Ok(t)) => t,
            Some(Err(e)) => {
                return Err(e);
            }
            None => {
                return Err(ParseError::eoe_msg(
                    curr_span,
                    ExpectedToken::AnyExpression,
                    "You must provide the return type of this definition.".to_string(),
                ));
            }
        };

        tokens
            .consume_token_or_error(vec![TokenKind::assign()])
            .map_err(|e| e.set_span_of_eof(curr_span))?;

        let ret_val = parse_expr(tokens, 0)?;

        tokens
            .consume_token_or_error(vec![TokenKind::semi_colon()])
            .map_err(|e| e.set_span_of_eof(curr_span))?;

        Ok(Stmt::Def(FuncDef::new(
            name, args, is_const,
            ret_type, ret_val,
            generics, curr_span,
        )))
    } else {
        let top_token = tokens.step().expect("Internal Compiler Error C9A5B07DC36");

        Err(ParseError::tok(
            top_token.kind.clone(),
            top_token.span,
            ExpectedToken::SpecificTokens(vec![
                TokenKind::keyword_use(),
                TokenKind::keyword_def(),
                TokenKind::Operator(OpToken::At),
            ]),
        ))
    }
}

// See test cases
pub fn parse_use(tokens: &mut TokenList, span: Span, is_top: bool) -> Result<Vec<Use>, ParseError> {
    let mut curr_paths: Vec<Use> = vec![];
    let mut curr_path: Vec<InternedString> = vec![];
    let mut curr_state = ParseUseState::IdentReady;
    let mut after_brace = false;
    let mut trailing_comma = false;

    loop {

        match curr_state {
            ParseUseState::IdentReady => match tokens.step() {
                Some(Token { kind, .. }) if kind.is_identifier() => {
                    curr_path.push(kind.unwrap_identifier());
                    curr_state = ParseUseState::IdentEnd;
                }
                Some(Token { kind: TokenKind::List(Delimiter::Brace, elements), span: brace_span }) => match parse_use(
                    &mut TokenList::from_vec(elements.to_vec()), span, false
                ) {
                    Ok(uses) => {

                        for use_case in uses.into_iter() {
                            curr_paths.push(use_case.push_front(&curr_path));
                        }

                        curr_path = vec![];
                        curr_state = ParseUseState::IdentEnd;
                        after_brace = true;
                    },
                    Err(e) => {
                        return Err(e.set_span_of_eof(*brace_span));
                    }
                }
                Some(Token { kind, span }) => {
                    return Err(ParseError::tok(
                        kind.clone(), *span,
                        ExpectedToken::SpecificTokens(vec![
                            TokenKind::dummy_identifier(),
                            TokenKind::List(Delimiter::Brace, vec![]),
                        ])
                    ));
                }
                None => {

                    if trailing_comma && !is_top {
                        return Ok(curr_paths);
                    }

                    else {
                        return Err(ParseError::eoe(
                            Span::dummy(),
                            ExpectedToken::SpecificTokens(vec![
                                TokenKind::dummy_identifier(),
                                TokenKind::List(Delimiter::Brace, vec![]),
                            ])
                        ));
                    }

                }
            }
            ParseUseState::IdentEnd => {
                let mut expected_tokens = vec![
                    TokenKind::comma(),
                ];

                if !after_brace {
                    expected_tokens.push(TokenKind::dot());
                    expected_tokens.push(TokenKind::keyword_as());
                }

                if is_top {
                    expected_tokens.push(TokenKind::semi_colon());
                }

                match tokens.step() {
                    Some(Token { kind: TokenKind::Operator(OpToken::Dot), span }) => {

                        if after_brace {
                            return Err(ParseError::tok(
                                TokenKind::dot(), *span,
                                ExpectedToken::SpecificTokens(expected_tokens)
                            ));
                        }

                        else {
                            curr_state = ParseUseState::IdentReady;
                        }

                    }
                    Some(Token { kind: TokenKind::Operator(OpToken::Comma), .. }) => {

                        if after_brace {
                            assert_eq!(curr_path.len(), 0, "Internal Compiler Error D10499A6C75");
                        }

                        else {
                            let alias = *curr_path.last().expect("Internal Compiler Error A57ABC5F7D7");
                            curr_paths.push(Use::new(curr_path, alias, span));
    
                            curr_path = vec![];
                        }

                        trailing_comma = true;
                        curr_state = ParseUseState::IdentReady;
                    }
                    Some(Token { kind: TokenKind::Operator(OpToken::SemiColon), span: colon_span }) => {

                        if curr_path.len() > 0 {
                            let alias = *curr_path.last().expect("Internal Compiler Error 52A9A947304");
                            curr_paths.push(Use::new(curr_path, alias, span));
                        }

                        if is_top {
                            return Ok(curr_paths);
                        }

                        else {
                            return Err(ParseError::tok(
                                TokenKind::semi_colon(), *colon_span,
                                ExpectedToken::SpecificTokens(expected_tokens)
                            ));
                        }
                    }
                    Some(Token { kind: TokenKind::Keyword(Keyword::As), span: as_span }) => {


                        if after_brace {
                            return Err(ParseError::tok(
                                TokenKind::Keyword(Keyword::As), *as_span,
                                ExpectedToken::SpecificTokens(expected_tokens)
                            ));
                        }

                        else {
                            curr_state = ParseUseState::AliasReady;
                        }

                    },
                    Some(Token { kind, span }) => {
                        return Err(ParseError::tok(
                            kind.clone(), *span,
                            ExpectedToken::SpecificTokens(expected_tokens)
                        ));
                    }
                    None => {

                        if is_top {
                            return Err(ParseError::eoe(
                                Span::dummy(),
                                ExpectedToken::SpecificTokens(expected_tokens)
                            ));
                        }

                        else {

                            if curr_path.len() > 0 {
                                let alias = *curr_path.last().expect("Internal Compiler Error 42A7CAA8F86");
                                curr_paths.push(Use::new(curr_path, alias, span));
                            }

                            return Ok(curr_paths);
                        }

                    }
                }
            }
            ParseUseState::AliasReady => match tokens.step() {
                Some(Token { kind, .. }) if kind.is_identifier() => {
                    curr_paths.push(Use::new(
                        curr_path,
                        kind.unwrap_identifier(),
                        span
                    ));

                    curr_path = vec![];
                    curr_state = ParseUseState::PathEnd;
                }
                Some(Token { kind, span }) => {
                    return Err(ParseError::tok(
                        kind.clone(), *span,
                        ExpectedToken::SpecificTokens(vec![TokenKind::dummy_identifier()])
                    ))
                }
                None => {
                    return Err(ParseError::eoe(
                        Span::dummy(),
                        ExpectedToken::SpecificTokens(vec![TokenKind::dummy_identifier()])
                    ))
                }
            }
            // expected: if is_top { [';', ','] } else { [',', '}'] }
            // `;` if is_top
            // None if !is_top
            // `,`
            ParseUseState::PathEnd => {

                let expected = if is_top {
                    vec![TokenKind::semi_colon(), TokenKind::comma()]
                } else {
                    vec![TokenKind::comma(), TokenKind::Operator(OpToken::ClosingCurlyBrace)]
                };

                match tokens.step() {
                    Some(Token { kind: TokenKind::Operator(OpToken::Comma), .. }) =>  {
                        trailing_comma = true;
                        curr_state = ParseUseState::IdentReady;
                    }
                    Some(Token { kind: TokenKind::Operator(OpToken::SemiColon), .. }) if is_top => {
                        return Ok(curr_paths);
                    }
                    Some(Token { kind, span }) => {
                        return Err(ParseError::tok(
                            kind.clone(), *span,
                            ExpectedToken::SpecificTokens(expected),
                        ));
                    }
                    None => {
                        if is_top {
                            return Err(ParseError::eoe(
                                Span::dummy(),
                                ExpectedToken::SpecificTokens(expected),
                            ))
                        }

                        else {
                            return Ok(curr_paths);
                        }
                    }
                }
            }
        }

        if after_brace && curr_state == ParseUseState::IdentReady {
            after_brace = false;
        }

        if trailing_comma && curr_state != ParseUseState::IdentReady {
            trailing_comma = false;
        }

    }
}

#[derive(PartialEq)]
enum ParseUseState {
    IdentReady,
    IdentEnd,
    AliasReady,
    PathEnd,
}
