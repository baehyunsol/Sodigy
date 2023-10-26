mod err;
mod num;
mod tests;
mod token;
mod session;

pub use err::LexError;
use num::{bin_to_dec, oct_to_dec, hex_to_dec};

use sodigy_data_structures::FixedVec;
use sodigy_err::{ErrorContext, SodigyError};
use sodigy_span::SpanPoint;
use sodigy_test::{sodigy_assert, TEST_MODE};

pub use session::LexSession;
pub use token::{Token, TokenKind};

pub enum LexState {
    Init,
    String { marker: u8, escaped: bool },
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

pub fn lex<const N: usize>(
    input: &[u8],
    mut index: usize,
    span_start: SpanPoint,  // span of `input[0]`
    session: &mut LexSession,
) -> Result<(), ()> {
    let mut curr_state = LexState::Init;
    let mut tmp_buf: FixedVec<u8, N> = FixedVec::init(0);
    let mut curr_token_span_start = span_start;

    loop {
        match input.get(index) {
            Some(c) => {
                let c = *c;

                match &mut curr_state {
                    LexState::Init => {
                        sodigy_assert!(tmp_buf.is_empty());

                        match c {
                            b'"' | b'\'' => {
                                curr_state = LexState::String { marker: c, escaped: false };
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

                                session.push_token(Token {
                                    kind,
                                    span: span_start.offset(index as i32).into_range(),
                                });
                            },
                        }
                    },
                    LexState::String { marker, escaped } => {
                        if *escaped {
                            *escaped = false;
                            tmp_buf.push(handle_escape_char(c));
                        }

                        else if c == b'\\' {
                            *escaped = true;
                        }

                        else if c == *marker {
                            let content = match String::from_utf8(tmp_buf.to_vec()) {
                                Ok(c) => c,
                                Err(_) => {
                                    session.push_error(LexError::invalid_utf8(curr_token_span_start.into_range()));
                                    return Err(());
                                },
                            };

                            session.push_token(Token {
                                kind: TokenKind::String {
                                    kind: (*marker).into(),
                                    content,
                                },
                                span: curr_token_span_start.extend(span_start.offset(index as i32 + 1)),
                            });

                            tmp_buf.flush();
                            curr_state = LexState::Init;
                        }

                        else {
                            tmp_buf.push(c);
                        }
                    },
                    LexState::Comment { kind, nest } => {
                        let comment_kind = *kind;
                        match comment_kind {
                            CommentKind::Single | CommentKind::Doc => {
                                if c == b'\n' {
                                    let content = match String::from_utf8(tmp_buf.to_vec()) {
                                        Ok(c) => c,
                                        Err(_) => {
                                            session.push_error(LexError::invalid_utf8(curr_token_span_start.into_range()));
                                            return Err(());
                                        },
                                    };

                                    session.push_token(Token {
                                        kind: TokenKind::Comment {
                                            kind: comment_kind,
                                            content,
                                        },
                                        span: curr_token_span_start.extend(span_start.offset(index as i32 + 1)),
                                    });

                                    tmp_buf.flush();
                                    curr_state = LexState::Init;
                                }

                                else {
                                    if let CommentKind::Doc = comment_kind {
                                        tmp_buf.push(c);
                                    }
                                }
                            },
                            CommentKind::Multi => {
                                if c == b'#' {
                                    if input.get(index + 1) == Some(&b'#')
                                    && input.get(index + 2) == Some(&b'!') {
                                        *nest += 1;
                                    }
                                }

                                else if is_multiline_comment_end(input, index) {
                                    *nest -= 1;

                                    if *nest == 0 {
                                        if let Err(_) = String::from_utf8(tmp_buf.to_vec()) {
                                            session.push_error(LexError::invalid_utf8(curr_token_span_start.into_range()));
                                            return Err(());
                                        }
                                        session.push_token(Token {
                                            kind: TokenKind::Comment {
                                                kind: CommentKind::Multi,
                                                content: String::new(),
                                            },
                                            span: curr_token_span_start.extend(span_start.offset(index as i32 + 2)),
                                        });

                                        index += 2;
                                        tmp_buf.flush();
                                        curr_state = LexState::Init;
                                    }
                                }

                                else {
                                    // we don't care about its content
                                    // tmp_buf.push(c);
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
                                kind: TokenKind::Identifier(session.intern_string(tmp_buf.to_vec())),
                                span: curr_token_span_start.extend(span_start.offset(index as i32)),
                            };

                            session.push_token(token);
                            tmp_buf.flush();
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
                                        curr_token_span_start.extend(span_start.offset(index as i32))
                                    ).set_expected_chars(
                                        b"0123456789_.eE".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                            _ => {
                                session.push_token(Token {
                                    kind: TokenKind::Number(tmp_buf.to_vec()),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.flush();
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
                                        curr_token_span_start.extend(span_start.offset(index as i32))
                                    ).set_expected_chars(
                                        b"bBoOxXeE.".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                            _ => {
                                session.push_token(Token {
                                    kind: TokenKind::Number(tmp_buf.to_vec()),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.flush();
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
                            b'_' => {
                                curr_state = LexState::NumberDecimalPoint;
                            },
                            b'e' | b'E' => {
                                tmp_buf.push(c);
                                curr_state = LexState::NumberExpInit;
                            },
                            b'.' => {
                                // likely to be reading `3..4` -> it's (`3`, `..`, `4`), not (`3.`, `.`, `4`)
                                tmp_buf.pop();
                                index -= 1;

                                session.push_token(Token {
                                    kind: TokenKind::Number(tmp_buf.to_vec()),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.flush();
                                curr_state = LexState::Init;
                                continue;
                            },
                            b'a'..=b'z' | b'A'..=b'Z' => {
                                session.push_error(
                                    LexError::unexpected_char(
                                        c as char,
                                        curr_token_span_start.extend(span_start.offset(index as i32))
                                    ).set_expected_chars(
                                        b"0123456789_eE".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                            _ => {
                                session.push_token(Token {
                                    kind: TokenKind::Number(tmp_buf.to_vec()),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.flush();
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
                                        curr_token_span_start.extend(span_start.offset(index as i32))
                                    ).set_expected_chars(
                                        b"0123456789_eE".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                            _ => {
                                session.push_token(Token {
                                    kind: TokenKind::Number(tmp_buf.to_vec()),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.flush();
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
                                        curr_token_span_start.extend(span_start.offset(index as i32))
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
                            b'0'..=b'9' => {},
                            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                                session.push_error(
                                    LexError::unexpected_char(
                                        c as char,
                                        curr_token_span_start.extend(span_start.offset(index as i32))
                                    ).set_expected_chars(
                                        b"0123456789".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                            _ => {
                                session.push_token(Token {
                                    kind: TokenKind::Number(tmp_buf.to_vec()),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.flush();
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
                                        curr_token_span_start.extend(span_start.offset(index as i32))
                                    ).set_expected_chars(
                                        b"01_".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                            _ => {
                                let result = match bin_to_dec(&tmp_buf.to_vec()[2..]) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        session.push_error(
                                            LexError::parse_num_error(e, curr_token_span_start.extend(span_start.offset(index as i32)))
                                        );
                                        return Err(());
                                    },
                                };

                                session.push_token(Token {
                                    kind: TokenKind::Number(result),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.flush();
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
                                        curr_token_span_start.extend(span_start.offset(index as i32))
                                    ).set_expected_chars(
                                        b"01234567_".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                            _ => {
                                let result = match oct_to_dec(&tmp_buf.to_vec()[2..]) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        session.push_error(
                                            LexError::parse_num_error(e, curr_token_span_start.extend(span_start.offset(index as i32)))
                                        );
                                        return Err(());
                                    },
                                };

                                session.push_token(Token {
                                    kind: TokenKind::Number(result),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.flush();
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
                                        curr_token_span_start.extend(span_start.offset(index as i32))
                                    ).set_expected_chars(
                                        b"0123456789aAbBcCdDeEfF_.".to_vec()
                                    ).set_err_context(
                                        ErrorContext::LexingNumericLiteral
                                    ).to_owned()
                                );
                                return Err(());
                            },
                            _ => {
                                let result = match hex_to_dec(&tmp_buf.to_vec()[2..]) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        session.push_error(
                                            LexError::parse_num_error(e, curr_token_span_start.extend(span_start.offset(index as i32)))
                                        );
                                        return Err(());
                                    },
                                };

                                session.push_token(Token {
                                    kind: TokenKind::Number(result),
                                    span: curr_token_span_start.extend(span_start.offset(index as i32)),
                                });
                                tmp_buf.flush();
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
                    LexState::String { marker, .. } => {
                        session.push_error(LexError::unfinished_string(marker.into(), curr_token_span_start.into_range()));
                        return Err(());
                    },
                    LexState::Identifier => {
                        let token = Token {
                            kind: TokenKind::Identifier(session.intern_string(tmp_buf.to_vec())),
                            span: curr_token_span_start.extend(span_start.offset(index as i32)),
                        };

                        session.push_token(token);
                    },
                    LexState::NumberInit
                    | LexState::NumberInitZero
                    | LexState::NumberDecimalPointInit
                    | LexState::NumberDecimalPoint
                    | LexState::NumberExp => {
                        session.push_token(Token {
                            kind: TokenKind::Number(tmp_buf.to_vec()),
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
                            session.push_token(Token {
                                kind: TokenKind::Number(tmp_buf.to_vec()),
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

                if TEST_MODE { session.get_tokens().iter().for_each(|token| token.assert_valid_span()); }
                return Ok(());
            }
        }

        index += 1;
    }
}

fn check_comment_kind(buf: &[u8], index: &mut usize) -> CommentKind {
    match (buf.get(*index + 1), buf.get(*index + 2)) {
        (Some(b'#'), Some(b'!')) => {
            *index += 2;

            CommentKind::Multi
        },
        (Some(b'#'), Some(b'>')) => {
            *index += 2;

            CommentKind::Doc
        },
        _ => CommentKind::Single,
    }
}

fn is_multiline_comment_end(buf: &[u8], index: usize) -> bool {
    matches!((buf.get(index), buf.get(index + 1), buf.get(index + 2)), (Some(b'!'), Some(b'#'), Some(b'#')))
}

// '\\' + c = result
fn handle_escape_char(c: u8) -> u8 {
    match c {
        b'n' => b'\n',
        b'r' => b'\r',
        b't' => b'\t',
        b'0' => b'\0',
        _ => c,
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

#[macro_export]
macro_rules! lex_flex {
    ($input: expr, $index: expr, $span_start: expr, $session: expr) => {
        lex_flex!(256, 1024, 4096, 16384, $input, $index, $span_start, $session)
    };
    ($n1: expr, $n2: expr, $n3: expr, $n4: expr, $input: expr, $index: expr, $span_start: expr, $session: expr) => {
        {
            let len = $input.len();

            if len < $n2 {
                if len < $n1 {
                    lex::<$n1>($input, $index, $span_start, $session)
                }

                else {
                    lex::<$n2>($input, $index, $span_start, $session)
                }
            }

            else {
                if len < $n3 {
                    lex::<$n3>($input, $index, $span_start, $session)
                }

                else {
                    lex::<$n4>($input, $index, $span_start, $session)
                }
            }
        }
    };
}
