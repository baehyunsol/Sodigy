// This file is Very Very experimental
// json -> sodigy transformation
// 1. `sodigy_lex` and `sodigy_parse` can digest json files
// 2. with very small transformations, we can convert a json file to a sodigy file (in Vec<TokenTree> level)
// 3. the converted file goes through normal compilation process
//
// The benefits are
// 1. it emits sodigy-style errors and warnings
// 2. less rust code, and more sodigy code
//
// It can only parse `sodigy.json`, not all json files. `sodigy.json` looks like below
// ```
// {
//     "macros": {
//         "foo": "path/to/foo"
//     },
//     "dependencies": {
//         "bar": "path/to/bar",
//         "baz": "path/to/baz",
//     },
// }
// ```
// The above code is converted to below
// ```
// let config = SodigyConfig.base() `macros [("foo", "path/to/foo")] `dependencies [("bar", "path/to/bar"), ("baz", "path/to/baz")];
// ```
// Since it uses sodigy's parser, the syntax is a bit more loose than the original json's.
// For example, it allows comments and trailing commas.

use crate::{Token, TokenKind};
use crate::error::AstError;
use crate::parse::parse_expr;
use crate::session::AstSession;
use crate::tokens::Tokens;
use sodigy_error::{ErrorContext, ExpectedToken, SodigyError};
use sodigy_intern::{intern_string, try_intern_short_string};
use sodigy_keyword::Keyword;
use sodigy_lex::QuoteKind;
use sodigy_parse::{Delim, Punct};
use sodigy_session::SodigySession;
use sodigy_span::SpanRange;

enum ParseState {
    ExpectTopLevelKey,
    ExpectTopLevelColon,
    ExpectTopLevelValue,
    ExpectTopLevelComma,
}

pub fn parse_config_file(tokens: &Vec<Token>, ast_session: &mut AstSession) -> Result<Vec<Token>, ()> {
    match tokens.get(0) {
        Some(Token {
            kind: TokenKind::Group {
                delim: Delim::Brace,
                tokens: inner_tokens,
                prefix: b'\0',
            },
            span,
        }) => {
            let mut result: Vec<Token> = vec![
                TokenKind::Keyword(Keyword::Let),
                TokenKind::Identifier(intern_string(b"config".to_vec())),
                TokenKind::Punct(Punct::Assign),
                TokenKind::Identifier(intern_string(b"SodigyConfig".to_vec())),
                TokenKind::Punct(Punct::Dot),
                TokenKind::Identifier(intern_string(b"base".to_vec())),
                TokenKind::Group { delim: Delim::Paren, prefix: b'\0', tokens: vec![] },
            ].into_iter().map(
                |token_kind| Token {
                    kind: token_kind,
                    span: SpanRange::dummy(0xc1cc089d),
                }
            ).collect();
            let mut curr_parse_state = ParseState::ExpectTopLevelKey;
            let mut last_token_span = *span;

            for token in inner_tokens.iter() {
                last_token_span = token.span;

                match curr_parse_state {
                    ParseState::ExpectTopLevelKey => match &token.kind {
                        TokenKind::String {
                            kind: QuoteKind::Double,
                            content,
                            is_binary: false,
                        } => {
                            result.push(Token {
                                kind: TokenKind::Punct(Punct::FieldModifier(*content)),
                                span: token.span,
                            });

                            curr_parse_state = ParseState::ExpectTopLevelColon;
                        },
                        _ => {
                            ast_session.push_error(AstError::unexpected_token(
                                token.clone(),
                                ExpectedToken::specific(TokenKind::String {
                                    kind: QuoteKind::Double,
                                    content: try_intern_short_string(b"...").unwrap(),
                                    is_binary: false,
                                }),
                            ));
                            return Err(());
                        },
                    },
                    ParseState::ExpectTopLevelColon => match &token.kind {
                        TokenKind::Punct(Punct::Colon) => {
                            curr_parse_state = ParseState::ExpectTopLevelValue;
                        },
                        _ => {
                            ast_session.push_error(AstError::unexpected_token(
                                token.clone(),
                                ExpectedToken::specific(TokenKind::Punct(Punct::Colon)),
                            ));
                            return Err(());
                        },
                    },
                    ParseState::ExpectTopLevelValue => match &token.kind {
                        TokenKind::Group {
                            delim: Delim::Brace,
                            prefix: b'\0',
                            tokens: inner_tokens,
                        } => match parse_top_level_value(inner_tokens, ast_session) {
                            Ok(tokens) => {
                                result.push(Token {
                                    kind: TokenKind::Group {
                                        delim: Delim::Bracket,
                                        prefix: b'\0',
                                        tokens,
                                    },
                                    span: token.span,
                                });

                                curr_parse_state = ParseState::ExpectTopLevelComma;
                            },
                            Err(_) => {
                                return Err(());
                            },
                        },
                        _ => {
                            ast_session.push_error(AstError::unexpected_token(
                                token.clone(),
                                ExpectedToken::specific(TokenKind::Group {
                                    delim: Delim::Brace,
                                    prefix: b'\0',
                                    tokens: vec![],
                                }),
                            ));
                            return Err(());
                        },
                    },
                    ParseState::ExpectTopLevelComma => match &token.kind {
                        TokenKind::Punct(Punct::Comma) => {
                            curr_parse_state = ParseState::ExpectTopLevelKey;
                        },
                        _ => {
                            ast_session.push_error(AstError::unexpected_token(
                                token.clone(),
                                ExpectedToken::specific(TokenKind::Punct(Punct::Comma)),
                            ));
                            return Err(());
                        },
                    },
                }
            }

            match curr_parse_state {
                ParseState::ExpectTopLevelColon => {
                    ast_session.push_error(AstError::unexpected_end(
                        last_token_span,
                        ExpectedToken::specific(TokenKind::Punct(Punct::Colon)),
                    ));
                },
                ParseState::ExpectTopLevelValue => {
                    ast_session.push_error(AstError::unexpected_end(
                        last_token_span,
                        ExpectedToken::specific(TokenKind::Group {
                            delim: Delim::Brace,
                            prefix: b'\0',
                            tokens: vec![],
                        }),
                    ));
                },
                ParseState::ExpectTopLevelComma
                | ParseState::ExpectTopLevelKey => {
                    // okay
                },
            }

            result.push(Token {
                kind: TokenKind::Punct(Punct::SemiColon),
                span: SpanRange::dummy(0x7e024c88),
            });

            if tokens.len() > 1 {
                ast_session.push_error(AstError::unexpected_token(
                    tokens[1].clone(),
                    ExpectedToken::nothing(),
                ));
            }

            // it assumes that the session doesn't have any error initially
            if !ast_session.has_error() {
                Ok(result)
            }

            else {
                Err(())
            }
        },
        Some(token) => {
            ast_session.push_error(AstError::unexpected_token(
                token.clone(),
                ExpectedToken::specific(TokenKind::Group {
                    delim: Delim::Brace,
                    prefix: b'\0',
                    tokens: vec![],
                }),
            ));

            Err(())
        },
        None => {
            ast_session.push_error(AstError::unexpected_end(
                SpanRange::dummy(0x3107cc6a),  // TODO: I want it to point to the config file
                ExpectedToken::Specific(vec![TokenKind::Group {
                    delim: Delim::Brace,
                    prefix: b'\0',
                    tokens: vec![],
                }]),
            ));

            Err(())
        }
    }
}

// `"foo": "path/to/foo", "bar": "path/to/bar"` -> `("foo", "path/to/foo"), ("bar", "path/to/bar")`
fn parse_top_level_value(tokens: &Vec<Token>, session: &mut AstSession) -> Result<Vec<Token>, ()> {
    let mut tokens_ = tokens.to_vec();
    let mut tokens_iter = Tokens::from_vec(&mut tokens_);
    let mut result = vec![];

    loop {
        if tokens_iter.is_finished() {
            return Ok(result);
        }

        let curr_span = tokens_iter.peek_span().unwrap();

        let key_start_index = tokens_iter.get_cursor();

        // key
        parse_expr(
            &mut tokens_iter,
            session,
            0,
            false,
            Some(ErrorContext::ParsingConfigFile),
            curr_span,
        )?;

        let key_end_index = tokens_iter.get_cursor();

        if let Err(mut e) = tokens_iter.consume(TokenKind::Punct(Punct::Colon)) {
            e.set_error_context(ErrorContext::ParsingConfigFile);
            session.push_error(e);

            return Err(());
        }

        let value_start_index = tokens_iter.get_cursor();

        // value
        parse_expr(
            &mut tokens_iter,
            session,
            0,
            false,
            Some(ErrorContext::ParsingConfigFile),
            curr_span,
        )?;

        let value_end_index = tokens_iter.get_cursor();

        // `"foo", "path/to/foo"` inside parenthesis
        let mut inner_tokens = vec![];

        for token in tokens[key_start_index..key_end_index].iter() {
            inner_tokens.push(token.clone());
        }

        inner_tokens.push(Token {
            kind: TokenKind::Punct(Punct::Comma),
            span: SpanRange::dummy(0x166f9b46),
        });

        for token in tokens[value_start_index..value_end_index].iter() {
            inner_tokens.push(token.clone());
        }

        result.push(Token {
            kind: TokenKind::Group {
                delim: Delim::Paren,
                prefix: b'\0',
                tokens: inner_tokens,
            },
            span: SpanRange::dummy(0x8a96dbee),
        });

        match tokens_iter.step() {
            Some(Token {
                kind: TokenKind::Punct(Punct::Comma),
                ..
            }) => {
                continue;
            },
            Some(token) => {
                session.push_error(AstError::unexpected_token(
                    token.clone(),
                    ExpectedToken::specific(TokenKind::Punct(Punct::Comma)),
                ));
                return Err(());
            },
            None => {
                return Ok(result);
            },
        }
    }
}
