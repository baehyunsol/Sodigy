use crate::arg::Arg;
use crate::flag::Flag;
use crate::error::ClapError;
use crate::lex::{Token, lex_cli};
use crate::token::TokenKind;
use sodigy_span::{SpanPoint, SpanRange};
use std::collections::HashMap;

#[derive(Debug)]
pub struct FlagWithArg {
    pub flag: Option<Flag>,
    pub flag_span: Option<SpanRange>,
    pub arg: Option<Arg>,
    pub arg_span: Option<SpanRange>,
}

pub fn parse_cli(code: &[u8], span_start: SpanPoint) -> Result<Vec<FlagWithArg>, Vec<ClapError>> {
    let tokens = lex_cli(code, span_start);
    let mut curr_flag = None;
    let mut curr_flag_span = None;
    let mut expected_token_kind = TokenKind::Optional(Box::new(TokenKind::Path));  // input file or another flag
    let mut result = vec![];
    let mut last_span = span_start.into_range();

    // tmp buffer for parsing `-L` args
    let mut library_tokens = vec![];

    for token in tokens.iter() {
        last_span = token.span;

        match &expected_token_kind {
            TokenKind::None => match Flag::try_parse(&token.buffer) {
                Some(flag) => {
                    result.push(FlagWithArg {
                        flag: curr_flag,
                        flag_span: curr_flag_span,
                        arg: None,
                        arg_span: None,
                    });
                    expected_token_kind = flag.arg_kind();
                    curr_flag_span = Some(token.span);
                    curr_flag = Some(flag);
                },
                None => {
                    if let Some(flag) = curr_flag {
                        result.push(FlagWithArg {
                            flag: curr_flag,
                            flag_span: curr_flag_span,
                            arg: None,
                            arg_span: None,
                        });
                    }

                    match TokenKind::Path.try_parse_arg(&token) {
                        Ok(input_path) => {
                            result.push(FlagWithArg {
                                // `curr_flag` is already consumed
                                flag: None,
                                flag_span: None,
                                arg: Some(input_path),
                                arg_span: Some(token.span),
                            });
                            curr_flag = None;
                            curr_flag_span = None;
                            expected_token_kind = TokenKind::Flag;
                        },
                        Err(e) => {
                            return Err(vec![e]);
                        },
                    }
                },
            },
            TokenKind::Flag => match Flag::try_parse(&token.buffer) {
                Some(flag) => {
                    expected_token_kind = flag.arg_kind();
                    curr_flag_span = Some(token.span);
                    curr_flag = Some(flag);
                },
                None => match TokenKind::Path.try_parse_arg(&token) {
                    Ok(input_path) => {
                        result.push(FlagWithArg {
                            // `curr_flag` is already consumed
                            flag: None,
                            flag_span: None,
                            arg: Some(input_path),
                            arg_span: Some(token.span),
                        });
                        curr_flag = None;
                        curr_flag_span = None;
                        expected_token_kind = TokenKind::Flag;
                    },
                    Err(e) => {
                        return Err(vec![e]);
                    },
                },
            },
            TokenKind::Optional(optional_arg_kind) => match Flag::try_parse(&token.buffer) {
                Some(flag) => {
                    result.push(FlagWithArg {
                        flag: curr_flag,
                        flag_span: curr_flag_span,
                        arg: None,
                        arg_span: None,
                    });
                    expected_token_kind = flag.arg_kind();
                    curr_flag_span = Some(token.span);
                    curr_flag = Some(flag);
                },
                None => match optional_arg_kind.try_parse_arg(&token) {
                    Ok(arg) => {
                        result.push(FlagWithArg {
                            flag: curr_flag,
                            flag_span: curr_flag_span,
                            arg: Some(arg),
                            arg_span: Some(token.span),
                        });
                        expected_token_kind = TokenKind::Flag;
                    },
                    // input_path is not allowed in this place -> that makes the syntax too nasty
                    Err(e) => {
                        return Err(vec![e]);
                    },
                },
            },
            TokenKind::Library => {
                match library_tokens.len() % 3 {
                    0 => if is_valid_library_name(&token.buffer) {
                        library_tokens.push(token);
                    } else if let Some(flag) = Flag::try_parse(&token.buffer) {
                        match parse_library_args(&library_tokens, curr_flag_span.unwrap()) {
                            Ok((arg, span)) => {
                                result.push(FlagWithArg {
                                    flag: curr_flag,
                                    flag_span: curr_flag_span,
                                    arg: Some(Arg::Library(arg)),
                                    arg_span: Some(span),
                                });
                                expected_token_kind = flag.arg_kind();
                                curr_flag_span = Some(token.span);
                                curr_flag = Some(flag);
                            },
                            Err(e) => {
                                return Err(vec![e]);
                            },
                        }
                    } else {
                        return Err(vec![ClapError::invalid_argument(
                            TokenKind::Flag,
                            &token.buffer,
                            token.span,
                        )]);
                    },
                    1 => if &token.buffer == b"=" {
                        library_tokens.push(token);
                    } else {
                        return Err(vec![ClapError::invalid_argument(
                            TokenKind::EqualSign,
                            &token.buffer,
                            token.span,
                        )]);
                    },
                    2 => {
                        library_tokens.push(token);
                    },
                    _ => unreachable!(),
                }
            },
            _ => match expected_token_kind.try_parse_arg(&token) {
                Ok(arg) => {
                    result.push(FlagWithArg {
                        flag: curr_flag,
                        flag_span: curr_flag_span,
                        arg: Some(arg),
                        arg_span: Some(token.span),
                    });
                    expected_token_kind = TokenKind::Flag;
                },
                Err(e) => {
                    return Err(vec![e]);
                },
            },
        }
    }

    match expected_token_kind {
        TokenKind::None
        | TokenKind::Optional(_) => match curr_flag.map(|f| (f, f.arg_kind())) {
            Some((flag, TokenKind::None | TokenKind::Optional(_))) => {
                result.push(FlagWithArg {
                    flag: Some(flag),
                    flag_span: curr_flag_span,
                    arg: None,
                    arg_span: None,
                });
            },
            _ => {},
        },
        TokenKind::Flag => {},
        _ => {
            return Err(vec![ClapError::no_arg(
                expected_token_kind,
                last_span,
            )]);
        },
    }

    if !result.is_empty() && result[0].flag.is_none() && result[0].arg.is_none() {
        result.remove(0);
    }

    if result.is_empty() {
        return Err(vec![ClapError::no_args_at_all()]);
    }

    Ok(result)
}

fn is_valid_library_name(b: &[u8]) -> bool {
    b.iter().all(
        |c| b'0' <= *c && *c <= b'9'
        || b'a' <= *c && *c <= b'z'
        || b'A' <= *c && *c <= b'Z'
        || *c == b'_' && *c == b'.'
    )
}

fn parse_library_args(
    tokens: &Vec<&Token>,
    flag_span: SpanRange,
) -> Result<(HashMap<String, String>, SpanRange), ClapError> {
    if tokens.is_empty() {
        return Err(ClapError::no_arg(
            TokenKind::Library,
            flag_span,
        ));
    }

    assert!(tokens.len() % 3 == 0);
    let mut result = HashMap::with_capacity(tokens.len() / 3);
    let mut index = 0;
    let mut entire_span = tokens[0].span;

    while index < tokens.len() {
        result.insert(
            String::from_utf8(tokens[index].buffer.to_vec()).map_err(
                |e| ClapError::invalid_utf8(tokens[index].span)
            )?,
            String::from_utf8(tokens[index + 2].buffer.to_vec()).map_err(
                |e| ClapError::invalid_utf8(tokens[index + 2].span)
            )?,
        );
        entire_span = entire_span.merge(tokens[index + 2].span);
        index += 3;
    }

    Ok((result, entire_span))
}
