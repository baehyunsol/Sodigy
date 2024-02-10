#![deny(unused_imports)]

mod endec;
mod error;
mod num;
mod tests;
mod token;
mod session;
mod warn;

pub use error::LexError;
use num::{bin_to_dec, oct_to_dec, hex_to_dec};

use log::info;
use sodigy_error::{ErrorContext, SodigyError};
use sodigy_session::SodigySession;
use sodigy_span::SpanPoint;

pub use session::LexSession;
pub use token::{Token, TokenKind};

/// This marker is used in order to differentiate '\\{}' and '\{}'.
/// It uses a special value that is never used in valid UTF-8 strings.
pub const FSTRING_START_MARKER: u8 = 251;

enum LexState {
    Init,
    String {
        marker: u8,
        escape: StringEscapeType,
        is_fstring: bool,  // has `\{` literal
    },
    Comment { kind: CommentKind, nest: usize },
    Identifier,

    // Number States
    NumberInit,              //   [1-9] + [0-9_]*
    NumberInitZero,          //   '0'
    NumberInitBin,           //   '0' + ('b' | 'B') + ('0' | '1' | '_')*
    NumberInitOct,           //   '0' + ('o' | 'O') + [0-7_]*
    NumberInitHex,           //   '0' + ('x' | 'X') + [0-9a-fA-F_]*
    NumberDecimalPointInit,  //   (NumberInit | NumberInitZero) + '.'
    NumberDecimalPoint,      //   NumberDecimalPointInit + [0-9_]*
    NumberExpInit,           //   (NumberInit | NumberInitZero | NumberDecimalPointInit | NumberDecimalPoint) + ('e' | 'E')
    NumberExp,               //    NumberExp + '-'? + [0-9]*
}

#[derive(Clone, Copy, Debug)]
pub enum CommentKind {
    Doc,
    Single,
    Multi,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QuoteKind {
    Double = b'\"' as isize,
    Single = b'\'' as isize,
}

impl From<u8> for QuoteKind {
    fn from(c: u8) -> Self {
        match c {
            b'"' => QuoteKind::Double,
            b'\'' => QuoteKind::Single,
            _ => unreachable!(),
        }
    }
}

enum StringEscapeType {
    None,
    Backslash,

    // f-strings can be nested
    FString(u16),
}

pub fn lex(
    input: &[u8],
    mut index: usize,
    span_start: SpanPoint,  // span of `input[0]`
    session: &mut LexSession,
) -> Result<(), ()> {
    info!(
        "sodigy_lex::lex(), first few chars are: {:?}",
        &input[index..(index + 8).min(input.len())],
    );

    let mut curr_state = LexState::Init;
    let mut tmp_buf = Vec::with_capacity(256);
    let mut curr_token_span_start = span_start;

    loop {
        match input.get(index) {
            Some(c) => {
                let c = *c;

                match &mut curr_state {
                    LexState::Init => {
                        debug_assert!(tmp_buf.is_empty());

                        match c {
                            b'"' | b'\'' => {
                                curr_state = LexState::String {
                                    marker: c,
                                    escape: StringEscapeType::None,
                                    is_fstring: false,
                                };
                                curr_token_span_start = span_start.offset(index as i32);
                            },
                            b'#' => {
                                curr_token_span_start = span_start.offset(index as i32);
                                curr_state = LexState::Comment {
                                    kind: check_comment_kind(input, &mut index),
                                    nest: 1,
                                };
                            },
                            b' ' | b'\n' | b'\t' => {
                                session.try_push_whitespace();
                            },
                            b'0'..=b'9' => {
                                if c == b'0' {
                                    curr_state = LexState::NumberInitZero;
                                } else {
                                    curr_state = LexState::NumberInit;
                                }

                                curr_token_span_start = span_start.offset(index as i32);
                                tmp_buf.push(c);
                            },
                            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                                curr_state = LexState::Identifier;
                                curr_token_span_start = span_start.offset(index as i32);
                                tmp_buf.push(c);
                            },
                            _ => {
                                let kind = TokenKind::try_lex_punct(c).map_err(
                                    |_| {
                                        let err_span = span_start.offset(index as i32).into_range();

                                        match try_get_char(input, index) {
                                            Some(c) => {
                                                LexError::unexpected_char(c, err_span)
                                            },
                                            None => {
                                                LexError::invalid_utf8(err_span)
                                            },
                                        }
                                    }
                                );

                                let kind = match kind {
                                    Ok(k) => k,
                                    Err(e) => {
                                        session.push_error(e);
                                        return Err(());
                                    }
                                };

                                session.push_result(Token {
                                    kind,
                                    span: span_start.offset(index as i32).into_range(),
                                });
                            },
                        }
                    },
                    LexState::String { marker, escape, is_fstring } => {
                        match escape {
                            StringEscapeType::Backslash => {
                                if c == b'{' {
                                    *escape = StringEscapeType::FString(1);
                                    *is_fstring = true;
                                    tmp_buf.push(FSTRING_START_MARKER);
                                }

                                else {
                                    *escape = StringEscapeType::None;

                                    match handle_escape_char(c) {
                                        Ok(c) => {
                                            tmp_buf.push(c);
                                        },
                                        Err(e) => {
                                            session.push_error(LexError::invalid_character_escape(
                                                e,
                                                curr_token_span_start.extend(span_start.offset(index as i32 + 1)),
                                            ));

                                            return Err(());
                                        },
                                    }
                                }
                            },
                            StringEscapeType::FString(stack) => {
                                if c == b'{' {
                                    *stack += 1;
                                }

                                else if c == b'}' {
                                    *stack -= 1;

                                    if *stack == 0 {
                                        *escape = StringEscapeType::None;
                                    }
                                }

                                tmp_buf.push(c);
                            },
                            StringEscapeType::None => {
                                if c == b'\\' {
                                    *escape = StringEscapeType::Backslash;
                                }

                                else if c == *marker {
                                    let content = match string_from_utf8(tmp_buf.clone()) {
                                        Ok(c) => c,
                                        Err(_) => {
                                            session.push_error(LexError::invalid_utf8(curr_token_span_start.into_range()));
                                            return Err(());
                                        },
                                    };

                                    session.push_result(Token {
                                        kind: TokenKind::String {
                                            kind: (*marker).into(),
                                            content,
                                            is_fstring: *is_fstring,
                                        },
                                        span: curr_token_span_start.extend(span_start.offset(index as i32 + 1)),
                                    });

                                    tmp_buf.clear();
                                    curr_state = LexState::Init;
                                }

                                else {
                                    tmp_buf.push(c);
                                }
                            },
                        }
                    },
                    LexState::Comment { kind, nest } => {
                        let comment_kind = *kind;
                        match comment_kind {
                            CommentKind::Single | CommentKind::Doc => {
                                if c == b'\n' {
                                    let content = match String::from_utf8(tmp_buf.clone()) {
                                        Ok(c) => c,
                                        Err(_) => {
                                            session.push_error(LexError::invalid_utf8(curr_token_span_start.into_range()));
                                            return Err(());
                                        },
                                    };

                                    session.push_result(Token {
                                        kind: TokenKind::Comment {
                                            kind: comment_kind,
                                            content,
                                        },
                                        span: curr_token_span_start.extend(span_start.offset(index as i32 + 1)),
                                    });

                                    tmp_buf.clear();
                                    curr_state = LexState::Init;
                                }

                                else {
                                    if let CommentKind::Doc = comment_kind {
                                        tmp_buf.push(c);
                                    }

                                    index += match curr_utf8_char_len(input, index) {
                                        Ok(i) => i,
                                        Err(_) => {
                                            session.push_error(LexError::invalid_utf8(curr_token_span_start.into_range()));
                                            return Err(());
                                        },
                                    };

                                    continue;
                                }
                            },
                            CommentKind::Multi => {
                                if c == b'#' {
                                    if input.get(index + 1) == Some(&b'!') {
                                        *nest += 1;
                                    }
                                }

                                else if is_multiline_comment_end(input, index) {
                                    *nest -= 1;

                                    if *nest == 0 {
                                        if let Err(_) = String::from_utf8(tmp_buf.clone()) {
                                            session.push_error(LexError::invalid_utf8(curr_token_span_start.into_range()));
                                            return Err(());
                                        }
                                        session.push_result(Token {
                                            kind: TokenKind::Comment {
                                                kind: CommentKind::Multi,
                                                content: String::new(),
                                            },
                                            span: curr_token_span_start.extend(span_start.offset(index as i32 + 1)),
                                        });

                                        index += 1;
                                        tmp_buf.clear();
                                        curr_state = LexState::Init;
                                    }
                                }

                                else {
                                    index += match curr_utf8_char_len(input, index) {
                                        Ok(i) => i,
                                        Err(_) => {
                                            session.push_error(LexError::invalid_utf8(curr_token_span_start.into_range()));
                                            return Err(());
                                        },
                                    };

                                    continue;
                                }
                            },
                        }
                    },
                    LexState::Identifier => {
                        if (b'a' <= c && c <= b'z')
                        || (b'A' <= c && c <= b'Z')
                        || (b'0' <= c && c <= b'9')
                        || b'_' == c {
                            tmp_buf.push(c);
                        }

                        else {
                            let token = Token {
                                kind: TokenKind::Identifier(session.intern_string(tmp_buf.clone())),
                                span: curr_token_span_start.extend(span_start.offset(index as i32)),
                            };

                            session.push_result(token);
                            tmp_buf.clear();
                            curr_state = LexState::Init;
                            continue;
                        }
                    },
                    LexState::NumberInit => {
                        match c {
                            b'0'..=b'9'  => {
                                tmp_buf.push(c);
                            },
                            b'_' => {},
                            b'.' => {
                                tmp_buf.push(c);
                                curr_state = LexState::NumberDecimalPointInit;
                            },
                            b'e' | b'E' => {
                                tmp_buf.push(c);
                                curr_state = LexState::NumberExpInit;
                            },
                            b'a'..=b'z' | b'A'..=b'Z' => {
                                session.push_error(
                                    LexError::unexpected_char(
                                        c as char,
                                        curr_token_span_start.extend(span_start.offset(index as i32)),
                                    ).set_expected_chars(
                                        b"0123456789_.eE".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                            _ => {
                                session.push_result(Token {
                                    kind: TokenKind::Number(tmp_buf.clone()),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.clear();
                                curr_state = LexState::Init;
                                continue;
                            },
                        }
                    },
                    LexState::NumberInitZero => {
                        match c {
                            b'b' | b'B' => {
                                tmp_buf.push(c);
                                curr_state = LexState::NumberInitBin;
                            },
                            b'o' | b'O' => {
                                tmp_buf.push(c);
                                curr_state = LexState::NumberInitOct;
                            },
                            b'x' | b'X' => {
                                tmp_buf.push(c);
                                curr_state = LexState::NumberInitHex;
                            },
                            b'e' | b'E' => {
                                tmp_buf.push(c);
                                curr_state = LexState::NumberExpInit;
                            },
                            b'.' => {
                                tmp_buf.push(c);
                                curr_state = LexState::NumberDecimalPointInit;
                            },
                            b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                                session.push_error(
                                    LexError::unexpected_char(
                                        c as char,
                                        span_start.offset(index as i32).into_range(),
                                    ).set_expected_chars(
                                        b"bBoOxXeE.".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                            _ => {
                                session.push_result(Token {
                                    kind: TokenKind::Number(tmp_buf.clone()),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.clear();
                                curr_state = LexState::Init;
                                continue;
                            },
                        }
                    },
                    LexState::NumberDecimalPointInit => {
                        match c {
                            b'0'..=b'9' => {
                                tmp_buf.push(c);
                                curr_state = LexState::NumberDecimalPoint;
                            },
                            // `3.e3` is not `3000.0`. It would try to find a field named `e3`.
                            b'.'
                            | b'a'..=b'z'
                            | b'A'..=b'Z'
                            | b'_' => {
                                // likely to be reading one of below
                                // - `3..4` -> it's (`3`, `..`, `4`), not (`3.`, `.`, `4`)
                                // - `3.pow(4)` -> it's (`3`, `.`, `pow`), not (`3.`, `pow`)
                                tmp_buf.pop().unwrap();
                                index -= 1;

                                session.push_result(Token {
                                    kind: TokenKind::Number(tmp_buf.clone()),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.clear();
                                curr_state = LexState::Init;
                                continue;
                            },
                            _ => {
                                session.push_result(Token {
                                    kind: TokenKind::Number(tmp_buf.clone()),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.clear();
                                curr_state = LexState::Init;
                                continue;
                            },
                        }
                    },
                    LexState::NumberDecimalPoint => {
                        match c {
                            b'0'..=b'9' => {
                                tmp_buf.push(c);
                            },
                            b'_' => {},
                            b'e' | b'E' => {
                                tmp_buf.push(c);
                                curr_state = LexState::NumberExpInit;
                            },
                            b'a'..=b'z' | b'A'..=b'Z' => {
                                session.push_error(
                                    LexError::unexpected_char(
                                        c as char,
                                        span_start.offset(index as i32).into_range(),
                                    ).set_expected_chars(
                                        b"0123456789_eE".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                            _ => {
                                session.push_result(Token {
                                    kind: TokenKind::Number(tmp_buf.clone()),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.clear();
                                curr_state = LexState::Init;
                                continue;
                            },
                        }
                    },
                    LexState::NumberExpInit => {
                        match c {
                            b'0'..=b'9' | b'-' => {
                                tmp_buf.push(c);
                                curr_state = LexState::NumberExp;
                            },
                            _ => {
                                session.push_error(
                                    LexError::unexpected_char(
                                        c as char,
                                        span_start.offset(index as i32).into_range(),
                                    ).set_expected_chars(
                                        b"0123456789-".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                        }
                    },
                    LexState::NumberExp => {
                        match c {
                            b'0'..=b'9' => {
                                tmp_buf.push(c);
                            },
                            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                                session.push_error(
                                    LexError::unexpected_char(
                                        c as char,
                                        span_start.offset(index as i32).into_range(),
                                    ).set_expected_chars(
                                        b"0123456789".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                            _ => {
                                session.push_result(Token {
                                    kind: TokenKind::Number(tmp_buf.clone()),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.clear();
                                curr_state = LexState::Init;
                                continue;
                            },
                        }
                    },
                    LexState::NumberInitBin => {
                        match c {
                            b'0' | b'1' => {
                                tmp_buf.push(c);
                            },
                            b'_' => {},
                            b'2'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' => {
                                session.push_error(
                                    LexError::unexpected_char(
                                        c as char,
                                        span_start.offset(index as i32).into_range(),
                                    ).set_expected_chars(
                                        b"01_".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                            _ => {
                                let result = match bin_to_dec(&tmp_buf[2..]) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        session.push_error(
                                            LexError::parse_num_error(
                                                e,
                                                curr_token_span_start.extend(span_start.offset(index as i32)),
                                            )
                                        );
                                        return Err(());
                                    },
                                };

                                session.push_result(Token {
                                    kind: TokenKind::Number(result),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.clear();
                                curr_state = LexState::Init;
                                continue;
                            },
                        }
                    },
                    LexState::NumberInitOct => {
                        match c {
                            b'0'..=b'7' => {
                                tmp_buf.push(c);
                            },
                            b'_' => {},
                            b'8' | b'9' | b'a'..=b'z' | b'A'..=b'Z' => {
                                session.push_error(
                                    LexError::unexpected_char(
                                        c as char,
                                        span_start.offset(index as i32).into_range(),
                                    ).set_expected_chars(
                                        b"01234567_".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                            _ => {
                                let result = match oct_to_dec(&tmp_buf[2..]) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        session.push_error(
                                            LexError::parse_num_error(
                                                e,
                                                curr_token_span_start.extend(span_start.offset(index as i32)),
                                            )
                                        );
                                        return Err(());
                                    },
                                };

                                session.push_result(Token {
                                    kind: TokenKind::Number(result),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.clear();
                                curr_state = LexState::Init;
                                continue;
                            },
                        }
                    },
                    LexState::NumberInitHex => {
                        match c {
                            b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {
                                tmp_buf.push(c);
                            },
                            b'_' => {},
                            b'g'..=b'z' | b'G'..=b'Z' => {
                                session.push_error(
                                    LexError::unexpected_char(
                                        c as char,
                                        span_start.offset(index as i32).into_range(),
                                    ).set_expected_chars(
                                        b"0123456789aAbBcCdDeEfF_.".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                            _ => {
                                let result = match hex_to_dec(&tmp_buf[2..]) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        session.push_error(
                                            LexError::parse_num_error(
                                                e,
                                                curr_token_span_start.extend(span_start.offset(index as i32)),
                                            )
                                        );
                                        return Err(());
                                    },
                                };

                                session.push_result(Token {
                                    kind: TokenKind::Number(result),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.clear();
                                curr_state = LexState::Init;
                                continue;
                            },
                        }
                    },
                }
            },
            None => {
                match curr_state {
                    LexState::Comment { kind: CommentKind::Multi, .. } => {
                        session.push_error(LexError::unfinished_comment(
                            curr_token_span_start.extend(
                                // it's 3 characters long
                                curr_token_span_start.offset(3)
                            )
                        ));
                        return Err(());
                    },
                    LexState::String { escape: StringEscapeType::FString(_), .. } => {
                        session.push_error(LexError::unfinished_fstring(curr_token_span_start.into_range()));
                        return Err(());
                    },
                    LexState::String { marker, .. } => {
                        session.push_error(LexError::unfinished_string(marker.into(), curr_token_span_start.into_range()));
                        return Err(());
                    },
                    LexState::Identifier => {
                        let token = Token {
                            kind: TokenKind::Identifier(session.intern_string(tmp_buf.clone())),
                            span: curr_token_span_start.extend(span_start.offset(index as i32)),
                        };

                        session.push_result(token);
                    },
                    LexState::NumberInit
                    | LexState::NumberInitZero
                    | LexState::NumberDecimalPointInit
                    | LexState::NumberDecimalPoint
                    | LexState::NumberExp => {
                        session.push_result(Token {
                            kind: TokenKind::Number(tmp_buf.clone()),
                            span: curr_token_span_start.extend(span_start.offset(index as i32)),
                        });
                    },
                    LexState::NumberInitBin
                    | LexState::NumberInitOct
                    | LexState::NumberInitHex => {
                        let expected = match curr_state {
                            LexState::NumberInitBin => b"01_".to_vec(),
                            LexState::NumberInitOct => b"01234567_".to_vec(),
                            LexState::NumberInitHex => b"0123456789aAbBcCdDeEfF_".to_vec(),
                            _ => unreachable!(),
                        };

                        if tmp_buf.len() == 2 {
                            session.push_error(
                                LexError::unfinished_num_literal(
                                    curr_token_span_start.extend(span_start.offset(index as i32))
                                ).set_expected_chars(
                                    expected
                                ).set_err_context(
                                    ErrorContext::LexingNumericLiteral
                                ).to_owned()
                            );
                            return Err(());
                        }

                        else {
                            session.push_result(Token {
                                kind: TokenKind::Number(tmp_buf.clone()),
                                span: curr_token_span_start.extend(span_start.offset(index as i32)),
                            });
                        }
                    },
                    LexState::NumberExpInit => {
                        session.push_error(
                            LexError::unfinished_num_literal(
                                curr_token_span_start.extend(span_start.offset(index as i32))
                            ).set_expected_chars(
                                b"0123456789".to_vec()
                            ).set_err_context(
                                ErrorContext::LexingNumericLiteral
                            ).to_owned()
                        );
                        return Err(());
                    },
                    LexState::Init | LexState::Comment { .. } => {}
                }

                // `.assert_valid_span` panics when the condition is not met
                debug_assert!(session.get_results().iter().for_each(|token| token.assert_valid_span()) == ());
                return Ok(());
            }
        }

        index += 1;
    }
}

fn check_comment_kind(buf: &[u8], index: &mut usize) -> CommentKind {
    match buf.get(*index + 1) {
        Some(b'!') => {
            *index += 1;

            CommentKind::Multi
        },
        Some(b'>') => {
            *index += 1;

            CommentKind::Doc
        },
        _ => CommentKind::Single,
    }
}

fn is_multiline_comment_end(buf: &[u8], index: usize) -> bool {
    matches!((buf.get(index), buf.get(index + 1)), (Some(b'!'), Some(b'#')))
}

// like `String::from_utf8` of Rust std, but it also allows `FSTRING_START_MARKER`
fn string_from_utf8(utf8: Vec<u8>) -> Result<String, ()> {
    let mut index = 0;

    while index < utf8.len() {
        if utf8[index] < 128 || utf8[index] == FSTRING_START_MARKER {
            index += 1;
            continue;
        }

        if let Some(c) = try_get_char(&utf8, index) {
            let c = c as u32;

            if c < 2048 {
                index += 2;
            }

            else if c < 65536 {
                index += 3;
            }

            else {
                index += 4;
            }
        }

        else {
            return Err(());
        }
    }

    unsafe { Ok(String::from_utf8_unchecked(utf8)) }
}

// '\\' + c = result
fn handle_escape_char(c: u8) -> Result<u8, u8> {
    match c {
        b'n' => Ok(b'\n'),
        b'r' => Ok(b'\r'),
        b't' => Ok(b'\t'),
        b'0' => Ok(b'\0'),
        b'\'' => Ok(b'\''),
        b'"' => Ok(b'"'),
        b'\\' => Ok(b'\\'),
        _ => Err(c),
    }
}

fn try_get_char(buf: &[u8], index: usize) -> Option<char> {
    let length = match buf.get(index) {
        Some(c) if *c < 128 => 1,
        Some(c) if *c < 192 => 0,
        Some(c) if *c < 224 => match buf.get(index + 1) {
            Some(c) if 128 <= *c && *c < 192 => 2,
            _ => 0,
        },
        Some(c) if *c < 240 => match buf.get(index + 1) {
            Some(c) if 128 <= *c && *c < 192 => match buf.get(index + 2) {
                Some(c) if 128 <= *c && *c < 192 => 3,
                _ => 0,
            },
            _ => 0,
        },
        Some(c) if *c < 248 => match buf.get(index + 1) {
            Some(c) if 128 <= *c && *c < 192 => match buf.get(index + 2) {
                Some(c) if 128 <= *c && *c < 192 => match buf.get(index + 3) {
                    Some(c) if 128 <= *c && *c < 192 => 4,
                    _ => 0,
                },
                _ => 0,
            },
            _ => 0,
        },
        Some(_) => 0,
        None => unreachable!(),
    };

    if length == 0 {
        None
    }

    else {
        match String::from_utf8(buf[index..(index + length)].to_vec()) {
            Ok(s) => s.chars().next(),
            Err(_) => None,
        }
    }
}

fn curr_utf8_char_len(buf: &[u8], index: usize) -> Result<usize, ()> {
    if let Some(c) = try_get_char(buf, index) {
        let c = c as u32;

        if c < 128 {
            Ok(1)
        }

        else if c < 2048 {
            Ok(2)
        }

        else if c < 65536 {
            Ok(3)
        }

        else {
            Ok(4)
        }
    }

    else {
        Err(())
    }
}
