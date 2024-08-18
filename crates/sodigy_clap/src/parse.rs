use crate::error::ClapError;
use crate::flag::Flag;
use crate::token::{Token, TokenKind, TokenValue};
use sodigy_files::{FileHash, global_file_session};
use sodigy_span::{SpanPoint, SpanRange};

// TODO: break lines if the input is too long
/// It converts command line arguments into a file, so that we can use spans.
pub fn into_file() -> (Vec<u8>, FileHash) {
    let args = std::env::args().map(
        |arg| if arg.chars().any(|c| c == '\n' || c == ' ' || c == '\'' || c == '\"') {
            format!("{arg:?}")
        } else {
            arg.to_string()
        }
    ).collect::<Vec<String>>();

    // first argument is the path to the binary
    // TODO: what if it isn't?
    let joined_args = (&args[1..]).join(" ");
    let file_session = unsafe { global_file_session() };

    let input: Vec<u8> = joined_args.into();
    let res = file_session.register_tmp_file(&input).unwrap();
    file_session.set_name_alias(res, "command_line_args".to_string());

    (input, res)
}

pub fn into_tokens(code: &[u8], span_start: SpanPoint) -> Result<Vec<Token>, Vec<ClapError>> {
    parse_cli(lex_cli(code, span_start))
}

struct TmpToken {
    buffer: Vec<u8>,
    span: SpanRange,
}

enum LexState {
    Init,
    Quoted { escaped: bool },
    Unquoted,
}

/// It seems inefficient to join the splitted tokens then split them again,
/// but that's the only way to use spans.
fn lex_cli(code: &[u8], span_start: SpanPoint) -> Vec<TmpToken> {
    let mut buffer = vec![];
    let mut tokens = vec![];
    let mut curr_state = LexState::Init;

    for (i, c) in code.iter().enumerate() {
        match &mut curr_state {
            LexState::Init => {
                match *c {
                    b' ' => {
                        continue;
                    },
                    b'\"' => {
                        curr_state = LexState::Quoted {
                            escaped: false,
                        };
                    },
                    b'=' => {  // let's tell the programmer that this is an error!
                        tokens.push(TmpToken {
                            buffer: vec![b'='],
                            span: span_start.offset(i as i32).into_range(),
                        });
                    },
                    c => {
                        buffer.push(c);
                        curr_state = LexState::Unquoted;
                    },
                }
            },
            LexState::Quoted { escaped } => {
                if *escaped {
                    buffer.push(*c);
                    *escaped = false;
                }

                else if *c == b'"' {
                    curr_state = LexState::Init;
                    tokens.push(TmpToken {
                        buffer: buffer.clone(),
                        span: span_start.offset(
                            (i - buffer.len()) as i32
                        ).extend(span_start.offset(i as i32)),
                    });
                    buffer.clear();
                }

                // TODO: can it handle `\n`?
                else if *c == b'\\' {
                    buffer.push(*c);
                    *escaped = true;
                }

                else {
                    buffer.push(*c);
                }
            },
            LexState::Unquoted => match *c {
                b' ' | b'=' => {
                    curr_state = LexState::Init;
                    tokens.push(TmpToken {
                        buffer: buffer.clone(),
                        span: span_start.offset(
                            (i - buffer.len()) as i32
                        ).extend(span_start.offset(i as i32)),
                    });
                    buffer.clear();

                    if *c == b'=' {  // let's tell the programmer that this is an error!
                        tokens.push(TmpToken {
                            buffer: vec![b'='],
                            span: span_start.offset(i as i32).into_range(),
                        });
                    }
                },
                c => {
                    buffer.push(c);
                },
            },
        }
    }

    if !buffer.is_empty() {
        tokens.push(TmpToken {
            span: span_start.offset(
                (code.len() - buffer.len()) as i32
            ).extend(span_start.offset(code.len() as i32)),
            buffer,
        });
    }

    tokens
}

enum ParseState {
    Init,
    ExpectArg(TokenKind),
}

fn parse_cli(tmp_tokens: Vec<TmpToken>) -> Result<Vec<Token>, Vec<ClapError>> {
    let mut errors = vec![];
    let mut curr_state = ParseState::Init;
    let mut tokens = Vec::with_capacity(tmp_tokens.len());

    // indices of assign operators in `tokens`.
    // it's later used to generate error messages
    let mut assign_operators = vec![];

    for token in tmp_tokens.into_iter() {
        match curr_state {
            ParseState::Init => {
                if token.buffer[0] == b'-' {
                    match Flag::try_parse(&token.buffer) {
                        Some(flag) => {
                            tokens.push(Token {
                                kind: TokenKind::Flag,
                                value: TokenValue::Flag(flag),
                                span: token.span,
                            });

                            match flag.param_type() {
                                TokenKind::None => {
                                    curr_state = ParseState::Init;
                                },
                                param => {
                                    curr_state = ParseState::ExpectArg(param);
                                },
                            }
                        },
                        None => {
                            errors.push(ClapError::invalid_flag(
                                token.buffer,
                                token.span,
                            ));
                        },
                    }
                }

                else {
                    match String::from_utf8(token.buffer.clone()) {
                        Ok(s) => {
                            if s == "=" {  // we made an extra token for error handling
                                assign_operators.push(tokens.len());
                                tokens.push(Token {
                                    kind: TokenKind::Error,
                                    value: TokenValue::None,
                                    span: token.span,
                                });
                            }

                            else {
                                tokens.push(Token {
                                    kind: TokenKind::Path,
                                    value: TokenValue::Path(s),
                                    span: token.span,
                                });
                            }
                        },
                        Err(_) => {
                            if &token.buffer[..] == b"=" {
                                assign_operators.push(tokens.len());
                                tokens.push(Token {
                                    kind: TokenKind::Error,
                                    value: TokenValue::None,
                                    span: token.span,
                                });
                            }

                            else {
                                errors.push(ClapError::invalid_utf8(token.span));
                            }
                        },
                    }
                }
            },
            ParseState::ExpectArg(param_type) => {
                match String::from_utf8(token.buffer.clone()) {
                    Ok(s) => match TokenValue::try_parse(&param_type, &s) {
                        Some(value) => {
                            tokens.push(Token {
                                kind: param_type,
                                value,
                                span: token.span,
                            });
                        },
                        None => {
                            if &token.buffer[..] == b"=" {
                                assign_operators.push(tokens.len());
                                tokens.push(Token {
                                    kind: TokenKind::Error,
                                    value: TokenValue::None,
                                    span: token.span,
                                });
                            }

                            else {
                                errors.push(ClapError::invalid_argument(
                                    param_type,
                                    &s,
                                    token.span,
                                ));
                            }
                        },
                    },
                    Err(_) => {
                        errors.push(ClapError::invalid_utf8(token.span));
                    },
                }

                curr_state = ParseState::Init;
            },
        }
    }

    if tokens.is_empty() && errors.is_empty() {
        errors.push(ClapError::no_args_at_all());
    }

    else if let ParseState::ExpectArg(kind) = curr_state {
        errors.push(ClapError::no_arg(kind, tokens.last().unwrap().span));
    }

    for assign_operator in assign_operators.into_iter() {
        errors.push(ClapError::assign_operator(
            if assign_operator == 0 { None } else { tokens.get(assign_operator - 1) },
            tokens[assign_operator].clone(),
            tokens.get(assign_operator + 1),
        ));
    }

    if errors.is_empty() {
        Ok(tokens)
    }

    else {
        Err(errors)
    }
}
