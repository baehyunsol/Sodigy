use crate::err::ParseError;
use crate::session::LocalParseSession;
use crate::span::Span;
use crate::token::{Delimiter, OpToken, Token, TokenKind};
use crate::utils::into_char;
use hmath::{ConversionError, Ratio};

pub fn lex_tokens(s: &[u8], session: &mut LocalParseSession) -> Result<Vec<Token>, ParseError> {
    let mut cursor = 0;
    let mut tokens = vec![];

    while let Ok(next_ind) = skip_whitespaces_and_comments(s, cursor, session) {
        cursor = next_ind;

        let (token, next_ind) = lex_token(s, cursor, session)?;
        tokens.push(token);
        cursor = next_ind;
    }

    Ok(tokens)
}

fn lex_token(
    s: &[u8],
    mut ind: usize,
    session: &mut LocalParseSession,
) -> Result<(Token, usize), ParseError> {
    let curr_span = Span::new(session.curr_file, ind);

    match s[ind] {
        b'\'' | b'"' => {
            let marker = s[ind];
            let mut buffer = vec![];

            let mut escaped = false;
            ind += 1;

            loop {
                if ind >= s.len() {
                    return Err(ParseError::eof(curr_span));
                }

                if !escaped && s[ind] == marker {
                    return Ok((
                        Token {
                            span: curr_span,
                            kind: TokenKind::String(session.intern_string(buffer)),
                        },
                        ind + 1,
                    ));
                }

                if !escaped && s[ind] == b'\\' {
                    escaped = true;
                } else if escaped {
                    if s[ind] == b'n' {
                        buffer.push(b'\n');
                    } else if s[ind] == b'r' {
                        buffer.push(b'\r');
                    } else if s[ind] == b't' {
                        buffer.push(b'\t');
                    } else {
                        buffer.push(s[ind]);
                    }

                    escaped = false;
                } else {
                    buffer.push(s[ind]);
                }

                ind += 1;
            }
        }
        b'0'..=b'9' => {
            let mut buffer = vec![];
            let mut dot_count = 0;

            while ind < s.len()
                && ((b'0' <= s[ind] && s[ind] <= b'9')
                    || (b'a' <= s[ind] && s[ind] <= b'z')
                    || (b'A' <= s[ind] && s[ind] <= b'Z')
                    || b'_' == s[ind]
                    || b'.' == s[ind])
            {
                buffer.push(s[ind]);

                // 1..2     -> range
                // 1.2..2.3 -> range
                // 1.2..    -> range
                // 1.0      -> ratio
                // 1.       -> ratio
                // 1..      -> range
                // 1...     -> syntax error
                // 1. ..    -> range
                if s[ind] == b'.' {
                    dot_count += 1;

                    if s.get(ind + 1) == Some(&b'.') {
                        break;
                    }
                }

                ind += 1;
            }

            // `1.2..` is valid (syntactically)
            if dot_count == 2 && buffer.last() == Some(&b'.') {
                buffer.pop().expect("Interal Compiler Error 6E339A1");
            }

            let string = String::from_utf8_lossy(&buffer).to_string();

            match Ratio::from_string(&string) {
                Ok(n) => Ok((
                    Token {
                        span: curr_span,
                        kind: TokenKind::Number(n),
                    },
                    ind,
                )),
                Err(e) => Err(match e {
                    ConversionError::NoData | ConversionError::UnexpectedEnd => {
                        ParseError::eof(curr_span)
                    }
                    ConversionError::InvalidChar(c) => ParseError::ch(c, curr_span),
                    _ => unreachable!("Internal Compiler Error 89CFCAA"),
                }),
            }
        }
        b'(' | b'{' | b'[' => {
            let marker = Delimiter::from(s[ind]);
            ind += 1;

            let end = marker.end();
            let mut data = vec![];

            loop {
                ind = skip_whitespaces_and_comments(s, ind, session)?;

                if s[ind] == end {
                    break;
                }

                let (e, new_ind) = lex_token(s, ind, session)?;
                ind = new_ind;
                data.push(Box::new(e));

                ind = skip_whitespaces_and_comments(s, ind, session)?;

                if s[ind] == end {
                    break;
                }
            }

            Ok((
                Token {
                    span: curr_span,
                    kind: TokenKind::List(marker, data),
                },
                ind + 1,
            ))
        }
        b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
            let mut buffer = vec![];

            while ind < s.len()
                && ((b'0' <= s[ind] && s[ind] <= b'9')
                    || (b'A' <= s[ind] && s[ind] <= b'Z')
                    || (b'a' <= s[ind] && s[ind] <= b'z')
                    || s[ind] == b'_')
            {
                buffer.push(s[ind]);
                ind += 1;
            }

            let string_index = session.intern_string(buffer);

            if let Some(k) = session.try_unwrap_keyword(string_index) {
                Ok((
                    Token {
                        span: curr_span,
                        kind: TokenKind::Keyword(k),
                    },
                    ind,
                ))
            } else {
                Ok((
                    Token {
                        span: curr_span,
                        kind: TokenKind::Identifier(string_index),
                    },
                    ind,
                ))
            }
        }
        b'+' | b'-' | b'*' | b'/' | b'%' | b'!' | b'=' | b'<' | b'>' | b',' | b'.' | b':'
        | b';' | b'&' | b'|' | b'@' => lex_op_tokens(s, ind, session),
        _ => Err(ParseError::ch(into_char(s, ind), curr_span)),
    }
}

// the first character is always valid!
fn lex_op_tokens(
    s: &[u8],
    ind: usize,
    session: &mut LocalParseSession,
) -> Result<(Token, usize), ParseError> {
    let curr_span = Span::new(session.curr_file, ind);

    if s[ind] == b'<' {
        if let Some(b'=') = s.get(ind + 1) {
            Ok((
                Token {
                    span: curr_span,
                    kind: TokenKind::Operator(OpToken::Le),
                },
                ind + 2,
            ))
        } else if let Some(b'>') = s.get(ind + 1) {
            Ok((
                Token {
                    span: curr_span,
                    kind: TokenKind::Operator(OpToken::Concat),
                },
                ind + 2,
            ))
        } else {
            Ok((
                Token {
                    span: curr_span,
                    kind: TokenKind::Operator(OpToken::Lt),
                },
                ind + 1,
            ))
        }
    } else if s[ind] == b'>' {
        if let Some(b'=') = s.get(ind + 1) {
            Ok((
                Token {
                    span: curr_span,
                    kind: TokenKind::Operator(OpToken::Ge),
                },
                ind + 2,
            ))
        } else {
            Ok((
                Token {
                    span: curr_span,
                    kind: TokenKind::Operator(OpToken::Gt),
                },
                ind + 1,
            ))
        }
    } else if s[ind] == b'=' {
        if let Some(b'=') = s.get(ind + 1) {
            Ok((
                Token {
                    span: curr_span,
                    kind: TokenKind::Operator(OpToken::Eq),
                },
                ind + 2,
            ))
        } else {
            Ok((
                Token {
                    span: curr_span,
                    kind: TokenKind::Operator(OpToken::Assign),
                },
                ind + 1,
            ))
        }
    } else if s[ind] == b'!' {
        if let Some(b'=') = s.get(ind + 1) {
            Ok((
                Token {
                    span: curr_span,
                    kind: TokenKind::Operator(OpToken::Ne),
                },
                ind + 2,
            ))
        } else {
            Ok((
                Token {
                    span: curr_span,
                    kind: TokenKind::Operator(OpToken::Not),
                },
                ind + 1,
            ))
        }
    } else if s[ind] == b',' {
        Ok((
            Token {
                span: curr_span,
                kind: TokenKind::Operator(OpToken::Comma),
            },
            ind + 1,
        ))
    } else if s[ind] == b':' {
        Ok((
            Token {
                span: curr_span,
                kind: TokenKind::Operator(OpToken::Colon),
            },
            ind + 1,
        ))
    } else if s[ind] == b';' {
        Ok((
            Token {
                span: curr_span,
                kind: TokenKind::Operator(OpToken::SemiColon),
            },
            ind + 1,
        ))
    } else if s[ind] == b'.' {
        if let Some(b'.') = s.get(ind + 1) {
            if let Some(b'.') = s.get(ind + 2) {
                Err(ParseError::ch_msg(
                    '.',
                    curr_span,
                    "`...` is not a valid syntax.
For a range operator following a real number, try `1. ..` or `(1.)..`
For consecutive range operators (which is likely a semantic error), try `(1..)..`"
                        .to_string(),
                ))
            } else {
                Ok((
                    Token {
                        span: curr_span,
                        kind: TokenKind::Operator(OpToken::DotDot),
                    },
                    ind + 2,
                ))
            }
        } else {
            Ok((
                Token {
                    span: curr_span,
                    kind: TokenKind::Operator(OpToken::Dot),
                },
                ind + 1,
            ))
        }
    } else if s[ind] == b'+' {
        Ok((
            Token {
                span: curr_span,
                kind: TokenKind::Operator(OpToken::Add),
            },
            ind + 1,
        ))
    } else if s[ind] == b'-' {
        Ok((
            Token {
                span: curr_span,
                kind: TokenKind::Operator(OpToken::Sub),
            },
            ind + 1,
        ))
    } else if s[ind] == b'*' {
        Ok((
            Token {
                span: curr_span,
                kind: TokenKind::Operator(OpToken::Mul),
            },
            ind + 1,
        ))
    } else if s[ind] == b'/' {
        Ok((
            Token {
                span: curr_span,
                kind: TokenKind::Operator(OpToken::Div),
            },
            ind + 1,
        ))
    } else if s[ind] == b'%' {
        Ok((
            Token {
                span: curr_span,
                kind: TokenKind::Operator(OpToken::Rem),
            },
            ind + 1,
        ))
    } else if s[ind] == b'@' {
        Ok((
            Token {
                span: curr_span,
                kind: TokenKind::Operator(OpToken::At),
            },
            ind + 1,
        ))
    } else if s[ind] == b'&' {
        if let Some(b'&') = s.get(ind + 1) {
            Ok((
                Token {
                    span: curr_span,
                    kind: TokenKind::Operator(OpToken::AndAnd),
                },
                ind + 2,
            ))
        } else {
            Ok((
                Token {
                    span: curr_span,
                    kind: TokenKind::Operator(OpToken::And),
                },
                ind + 1,
            ))
        }
    } else if s[ind] == b'|' {
        if let Some(b'|') = s.get(ind + 1) {
            Ok((
                Token {
                    span: curr_span,
                    kind: TokenKind::Operator(OpToken::OrOr),
                },
                ind + 2,
            ))
        } else {
            Ok((
                Token {
                    span: curr_span,
                    kind: TokenKind::Operator(OpToken::Or),
                },
                ind + 1,
            ))
        }
    } else {
        unreachable!("Internal Compiler Error 71B2472: {:?}", s[ind])
    }
}

// initial `ind` must either be (1) first character of a value or a delimiter, (2) whitespace, or (3) start of a comment
// the returned value is always either be (1) EOF, or (2) first character of a value or a delimiter
// if the initial `ind` or the returned `ind` is not inside `s`, it returns `ParseError::UnexpectedEof`
fn skip_whitespaces_and_comments(
    s: &[u8],
    mut ind: usize,
    session: &mut LocalParseSession,
) -> Result<usize, ParseError> {
    let curr_span = Span::new(session.curr_file, ind);

    while ind < s.len() {
        if s[ind] == b' ' || s[ind] == b'\n' || s[ind] == b'\r' || s[ind] == b'\t' {
            ind += 1;
        } else if s[ind] == b'#' {
            ind += 1;

            while ind < s.len() && s[ind] != b'\n' {
                ind += 1;
            }
        } else {
            return Ok(ind);
        }
    }

    Err(ParseError::eof(curr_span))
}
