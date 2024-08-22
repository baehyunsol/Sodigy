use sodigy_files::{FileHash, global_file_session};
use sodigy_span::{SpanPoint, SpanRange};

pub struct Token {
    pub buffer: Vec<u8>,
    pub span: SpanRange,
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
        |arg| if arg.chars().any(
            |c| c == '\n' || c == ' '
            || c == '\'' || c == '\"'
            || c == '\r' || c == '\t'
            || c == '\0' || c == '\\'
        ) {
            format!("{arg:?}")
        } else {
            arg.to_string()
        }
    ).collect::<Vec<String>>();

    // TODO: I want the spans in the error messages to show the path to the binary, or at least a string "sodigy"
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
pub fn lex_cli(code: &[u8], span_start: SpanPoint) -> Vec<Token> {
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
                        buffer: unescape_rust_string_literal(&buffer),
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

fn unescape_rust_string_literal(buffer: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(buffer.len());
    let mut is_escaped = false;

    for c in buffer.iter() {
        if is_escaped {
            let cc = match *c {
                b'r' => b'\r',
                b'n' => b'\n',
                b't' => b'\t',
                b'0' => b'\0',
                _ => *c,
            };

            result.push(cc);
            is_escaped = false;
        }

        else {
            if *c == b'\\' {
                is_escaped = true;
            }

            else {
                result.push(*c);
            }
        }
    }

    result
}
