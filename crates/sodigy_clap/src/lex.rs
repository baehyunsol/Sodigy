use sodigy_files::{FileHash, global_file_session};

pub struct Token {
    buffer: Vec<u8>,
    span: SpanRange,
}

enum LexState {
    Init,
    Quoted { escaped: bool },
    Unquoted,
}

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
    let joined_args = (&args[1..]).join(" ");
    let file_session = unsafe { global_file_session() };

    let input: Vec<u8> = joined_args.into();
    let res = file_session.register_tmp_file(&input).unwrap();
    file_session.set_name_alias(res, "command_line_args".to_string());

    (input, res)
}

/// It seems inefficient to join the splitted tokens then split them again,
/// but that's the only way to use spans.
fn lex_cli(code: &[u8], span_start: SpanPoint) -> Vec<Token> {
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
                    b'=' => {
                        tokens.push(Token {
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
                    tokens.push(Token {
                        buffer: buffer.clone(),
                        span: span_start.offset(
                            (i - buffer.len()) as i32
                        ).extend(span_start.offset(i as i32)),
                    });
                    buffer.clear();
                }

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
                    tokens.push(Token {
                        buffer: buffer.clone(),
                        span: span_start.offset(
                            (i - buffer.len()) as i32
                        ).extend(span_start.offset(i as i32)),
                    });
                    buffer.clear();

                    if *c == b'=' {
                        tokens.push(Token {
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
        tokens.push(Token {
            span: span_start.offset(
                (code.len() - buffer.len()) as i32
            ).extend(span_start.offset(code.len() as i32)),
            buffer,
        });
    }

    tokens
}
