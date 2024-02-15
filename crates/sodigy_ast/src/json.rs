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
use sodigy_error::ExpectedToken;
use sodigy_intern::try_intern_short_string;
use sodigy_lex::QuoteKind;
use sodigy_parse::{Delim, Punct};
use sodigy_span::SpanRange;

enum ParseState {
    ExpectTopLevelKey,
    ExpectTopLevelColon,
    ExpectTopLevelValue,
    ExpectTopLevelComma,
}

pub fn parse_config_file(tokens: &Vec<Token>) -> Result<Vec<Token>, Vec<AstError>> {
    match tokens.get(0) {
        Some(Token {
            kind: TokenKind::Group {
                delim: Delim::Brace,
                tokens: inner_tokens,
                prefix: b'\0',
            },
            span,
        }) => {
            let mut result = vec![
                // TODO
                // Keyword("let"),
                // Ident("config"),
                // Punct("="),
                // Ident("SodigyConfig"),
                // Punct("."),
                // Ident("base"),
                // Group { delim: Paren, tokens: [] },
            ];
            let mut errors = vec![];
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
                            errors.push(AstError::unexpected_token(
                                token.clone(),
                                ExpectedToken::specific(TokenKind::String {
                                    kind: QuoteKind::Double,
                                    content: try_intern_short_string(b"...").unwrap(),
                                    is_binary: false,
                                }),
                            ));
                            break;
                        },
                    },
                    ParseState::ExpectTopLevelColon => match &token.kind {
                        TokenKind::Punct(Punct::Colon) => {
                            curr_parse_state = ParseState::ExpectTopLevelValue;
                        },
                        _ => {
                            errors.push(AstError::unexpected_token(
                                token.clone(),
                                ExpectedToken::specific(TokenKind::Punct(Punct::Colon)),
                            ));
                            break;
                        },
                    },
                    ParseState::ExpectTopLevelValue => match &token.kind {
                        TokenKind::Group {
                            delim: Delim::Brace,
                            prefix: b'\0',
                            tokens: inner_tokens,
                        } => match parse_top_level_value(inner_tokens) {
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
                            Err(errors_res) => {
                                for error in errors_res.into_iter() {
                                    errors.push(error);
                                }

                                break;
                            },
                        },
                        _ => {
                            errors.push(AstError::unexpected_token(
                                token.clone(),
                                ExpectedToken::specific(TokenKind::Group {
                                    delim: Delim::Brace,
                                    prefix: b'\0',
                                    tokens: vec![],
                                }),
                            ));
                            break;
                        },
                    },
                    ParseState::ExpectTopLevelComma => match &token.kind {
                        TokenKind::Punct(Punct::Comma) => {
                            curr_parse_state = ParseState::ExpectTopLevelValue;
                        },
                        _ => {
                            errors.push(AstError::unexpected_token(
                                token.clone(),
                                ExpectedToken::specific(TokenKind::Punct(Punct::Comma)),
                            ));
                            break;
                        },
                    },
                }
            }

            match curr_parse_state {
                ParseState::ExpectTopLevelColon => {
                    errors.push(AstError::unexpected_end(
                        last_token_span,
                        ExpectedToken::specific(TokenKind::Punct(Punct::Colon)),
                    ));
                },
                ParseState::ExpectTopLevelValue => {
                    errors.push(AstError::unexpected_end(
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

            // result.push(Punct(";"));

            if tokens.len() > 1 {
                errors.push(AstError::unexpected_token(
                    tokens[1].clone(),
                    ExpectedToken::nothing(),
                ));
            }

            if errors.is_empty() {
                Ok(result)
            }

            else {
                Err(errors)
            }
        },
        Some(token) => Err(vec![
            AstError::unexpected_token(
                token.clone(),
                ExpectedToken::specific(TokenKind::Group {
                    delim: Delim::Brace,
                    prefix: b'\0',
                    tokens: vec![],
                }),
            ),
        ]),
        None => Err(vec![
            AstError::unexpected_end(
                SpanRange::dummy(0x3107cc6a),  // TODO: I want it to point to the config file
                ExpectedToken::Specific(vec![TokenKind::Group {
                    delim: Delim::Brace,
                    prefix: b'\0',
                    tokens: vec![],
                }]),
            ),
        ]),
    }
}

enum InnerParseState {
    ExpectKey,
    ExpectColon,
}

// `"foo": "path/to/foo", "bar": "path/to/bar"` -> `("foo", "path/to/foo"), ("bar", "path/to/bar")`
// TODO: how about just using `parse_expr`?
fn parse_top_level_value(tokens: &Vec<Token>) -> Result<Vec<Token>, Vec<AstError>> {
    if tokens.is_empty() {
        return Ok(vec![]);
    }

    let mut curr_parse_state = InnerParseState::ExpectKey;
    let mut curr_key = Token::new_punct();

    for token in tokens.iter() {
        match curr_parse_state {
            InnerParseState::ExpectKey => {
                curr_key = token.clone();
                curr_parse_state = InnerParseState::ExpectColon;
            },
            InnerParseState::ExpectColon => match &token.kind {},
            InnerParseState::ExpectValue => {},
            InnerParseState::ExpectComma => match &token.kind {},
        }
    }

    match curr_parse_state {}
}
