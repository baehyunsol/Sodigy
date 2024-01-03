use crate::error::ClapError;
use crate::flag::Flag;
use crate::token::{Token, TokenKind, TokenValue};
use sodigy_files::{FileHash, global_file_session};
use sodigy_span::SpanPoint;

// TODO: if the programmer uses `=` after a flag, tell them to remove that

// TODO: break lines if the input is too long
/// It converts command line arguments into a file, so that we can use spans.
pub fn into_file() -> (Vec<u8>, FileHash) {
    let mut args = std::env::args().map(
        |arg| if arg.chars().any(|c| c == '\n' || c == ' ') {
            format!("{arg:?}")
        } else {
            arg.to_string()
        }
    ).collect::<Vec<String>>();

    // a simple hack: it makes the parser handles the last token
    args.push(String::from(" "));

    let joined_args = args.join(" ");
    let file_session = unsafe { global_file_session() };

    let input: Vec<u8> = joined_args.into();
    let res = file_session.register_tmp_file(input.clone());
    file_session.set_name_alias(res, "command_line_args".to_string());

    (input, res)
}

enum LexState {
    Quoted { escaped: bool },
    NotQuoted,
    TokenEnded,
}

enum ParseState {
    Ignore,  // ignore the first token (usually the path of the binary)
    FlagOrInput,
    Argument(TokenKind),
}

/// It seems inefficient to join the splitted tokens then split them again,
/// but that's the only way to use spans.
pub fn into_tokens(code: &[u8], span_start: SpanPoint) -> Result<Vec<Token>, Vec<ClapError>> {
    let mut buf = vec![];
    let mut lex_state = LexState::TokenEnded;
    let mut parse_state = ParseState::Ignore;
    let mut tokens = vec![];
    let mut errors = vec![];

    for (i, c) in code.iter().enumerate() {
        if let LexState::TokenEnded = lex_state {
            if *c == b'"' {
                lex_state = LexState::Quoted { escaped: false };
            }

            else {
                lex_state = LexState::NotQuoted;
                buf.push(*c);
            }

            continue;
        }

        let buf_end = match &mut lex_state {
            LexState::NotQuoted => {
                if *c == b' ' {
                    true
                }

                else {
                    buf.push(*c);
                    false
                }
            },
            LexState::Quoted { escaped } => {
                if *escaped {
                    buf.push(*c);
                    *escaped = false;
                    false
                }

                else if *c == b'"' {
                    true
                }

                else if *c == b'\\' {
                    buf.push(*c);
                    *escaped = true;
                    false
                }

                else {
                    buf.push(*c);
                    false
                }
            },
            LexState::TokenEnded => unreachable!(),
        };

        if buf_end {
            lex_state = LexState::TokenEnded;
            let curr_span = span_start.offset(
                (i - buf.len()) as i32
            ).extend(span_start.offset(i as i32));

            if buf.is_empty() {
                continue;
            }

            match parse_state {
                ParseState::Ignore => {
                    parse_state = ParseState::FlagOrInput;
                },
                ParseState::FlagOrInput => {
                    if buf[0] == b'-' {
                        match Flag::try_parse(&buf) {
                            Some(flag) => {
                                tokens.push(Token {
                                    kind: TokenKind::Flag,
                                    value: TokenValue::Flag(flag),
                                    span: curr_span,
                                });

                                let param_type = flag.param_type();

                                if let TokenKind::None = param_type {
                                    parse_state = ParseState::FlagOrInput;
                                }

                                else {
                                    parse_state = ParseState::Argument(param_type);
                                }
                            },
                            None => {
                                errors.push(ClapError::invalid_flag(
                                    buf.to_vec(), curr_span,
                                ));
                            },
                        }
                    }

                    else {
                        match String::from_utf8(buf.clone()) {
                            Ok(s) => {
                                tokens.push(Token {
                                    kind: TokenKind::Path,
                                    value: TokenValue::Path(s),
                                    span: curr_span,
                                });
                            },
                            Err(_) => {
                                errors.push(ClapError::invalid_utf8(curr_span));
                            },
                        }
                    }
                },
                ParseState::Argument(input_kind) => {
                    match String::from_utf8(buf.clone()) {
                        Ok(s) => match TokenValue::try_parse(&input_kind, &s) {
                            Some(value) => {
                                tokens.push(Token {
                                    kind: input_kind,
                                    value,
                                    span: curr_span,
                                });
                            },
                            None => {
                                errors.push(ClapError::invalid_argument(input_kind, &s, curr_span));
                            },
                        },
                        Err(_) => {
                            errors.push(ClapError::invalid_utf8(curr_span));
                        },
                    }

                    parse_state = ParseState::FlagOrInput;
                },
            }

            buf.clear();
        }
    }

    if tokens.is_empty() && errors.is_empty() {
        errors.push(ClapError::no_args_at_all());
    }

    if let ParseState::Argument(kind) = parse_state {
        errors.push(ClapError::no_arg(kind, span_start.offset(code.len() as i32 - 1).into_range()));
    }

    if errors.is_empty() {
        Ok(tokens)
    }

    else {
        Err(errors)
    }
}
