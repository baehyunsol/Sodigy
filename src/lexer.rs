use crate::err::{ExpectedToken, ParseError};
use crate::session::LocalParseSession;
use crate::span::Span;
use crate::token::{Delimiter, OpToken, Token, TokenKind};
use crate::utils::{bytes_to_string, bytes_to_v32, into_char, v32_to_bytes};
use hmath::{ConversionError, Ratio};

pub fn lex_tokens(s: &[u8], session: &mut LocalParseSession) -> Result<Vec<Token>, ParseError> {
    let mut cursor = 0;
    let mut tokens = vec![];

    while let Some(next_ind) = skip_whitespaces_and_comments(s, cursor, session) {
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
                    return Err(ParseError::eof_msg(curr_span, String::from("Unexpected EOF while parsing a string literal!")));
                }

                if !escaped && s[ind] == marker {
                    return Ok((
                        Token {
                            span: curr_span,
                            kind: TokenKind::String(
                                bytes_to_v32(&buffer).map_err(
                                    |e| e.set_ind_and_fileno(curr_span)
                                )?
                            ),
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
                buffer.pop().expect("Internal Compiler Error 6E339A1");
            }

            let string = bytes_to_string(&buffer);

            match Ratio::from_string(&string) {
                Ok(n) => Ok((
                    Token {
                        span: curr_span,
                        kind: TokenKind::Number(n),
                    },
                    ind,
                )),
                Err(e) => Err(match e {
                    ConversionError::NoData
                    | ConversionError::UnexpectedEnd => ParseError::eof(curr_span),
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
                ind = skip_whitespaces_and_comments(s, ind, session).ok_or(
                    ParseError::eoe_msg(
                        curr_span,
                        ExpectedToken::SpecificTokens(vec![marker.closing_token_kind()]),
                        format!("`{marker}` is not closed properly!"),
                    )
                )?;

                if s[ind] == end {
                    break;
                }

                let (e, new_ind) = lex_token(s, ind, session)?;
                ind = new_ind;
                data.push(Box::new(e));

                ind = skip_whitespaces_and_comments(s, ind, session).ok_or(
                    ParseError::eoe_msg(
                        curr_span,
                        ExpectedToken::SpecificTokens(vec![marker.closing_token_kind()]),
                        format!("`{marker}` is not closed properly!"),
                    )
                )?;

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

            // byte string literals and formatted string literals
            if s[ind] == b'b' || s[ind] == b'f' {
                match s.get(ind + 1) {
                    Some(c) if *c == b'\'' || *c == b'"' => {
                        let (string_literal, end_index) = lex_token(s, ind + 1, session)?;

                        return if s[ind] == b'b' {
                            Ok((
                                string_to_bytes(string_literal)?,
                                end_index,
                            ))
                        }

                        else {
                            Ok((
                                string_to_formatted(string_literal, session)?,
                                end_index,
                            ))
                        };
                    },
                    _ => {}
                }
            }

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
        | b';' | b'&' | b'|' | b'@' | b'\\' => lex_op_tokens(s, ind, session),
        _ => Err(ParseError::ch(into_char(s, ind).map_err(|e| e.set_ind_and_fileno(curr_span))?, curr_span)),
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
                    kind: TokenKind::dot(),
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
    } else if s[ind] == b'\\' {
        Ok((
            Token {
                span: curr_span,
                kind: TokenKind::Operator(OpToken::BackSlash),
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
// if the initial `ind` or the returned `ind` is not inside `s`, it returns None
fn skip_whitespaces_and_comments(
    s: &[u8],
    mut ind: usize,
    session: &mut LocalParseSession,
) -> Option<usize> {
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
            return Some(ind);
        }
    }

    None
}

fn string_to_bytes(t: Token) -> Result<Token, ParseError> {
    // t.span points to `"` of `b"`, but it should point to `b`.
    let span = t.span.backward(1).expect("Internal Compiler Error FEDF1CB");

    Ok(Token {
        kind: TokenKind::Bytes(v32_to_bytes(t.kind.unwrap_string())),
        span,
    })
}

fn string_to_formatted(t: Token, session: &mut LocalParseSession) -> Result<Token, ParseError> {
    // t.span points to `"` of `f"`, but it should point to `f`.
    let span = t.span.backward(1).expect("Internal Compiler Error C30F25D");

    let string = t.kind.unwrap_string();
    let mut curr_state = FormatStringParseState::String;
    let mut tmp_buffer = vec![];
    let mut buffer = vec![];
    let mut curr_start_span = 0;
    let mut nested_braces = 0;

    for (i, c) in string.iter().enumerate() {

        match curr_state {
            FormatStringParseState::String => {

                if *c == '{' as u32 {

                    if !tmp_buffer.is_empty() {
                        buffer.push(vec![Token {
                            kind: TokenKind::String(tmp_buffer),
                            span: span.forward(curr_start_span + 2),  // 2 for `f"`
                        }]);
                    }

                    curr_start_span = i;
                    tmp_buffer = vec![];
                    curr_state = FormatStringParseState::Value;
                    nested_braces = 1;
                }

                else {
                    tmp_buffer.push(*c);
                }

            }
            FormatStringParseState::Value => {

                if *c == '}' as u32 {
                    nested_braces -= 1;

                    if nested_braces == 0 {
                        let value_string = v32_to_bytes(&tmp_buffer);
                        let inner_value = lex_tokens(&value_string, session)
                            .map(|tokens| set_span_of_formatted_string(tokens, span.forward(curr_start_span + 3)))
                            .map_err(|error| set_span_of_formatted_string_err(error, span.forward(curr_start_span + 3)))?;

                        if inner_value.is_empty() {
                            return Err(ParseError::eoe(
                                span.forward(curr_start_span + 2),
                                ExpectedToken::AnyExpression,
                            ));
                        }

                        buffer.push(inner_value);
                        curr_start_span = i + 1;
                        tmp_buffer = vec![];
                        curr_state = FormatStringParseState::String;
                    }
                }

                // it allows `f"{{{3}}}"`
                else if *c == '{' as u32 {
                    nested_braces += 1;
                }

                else {
                    tmp_buffer.push(*c);
                }

            }
        }

    }

    if !tmp_buffer.is_empty() {

        match curr_state {
            FormatStringParseState::String => {
                buffer.push(vec![Token {
                    kind: TokenKind::String(tmp_buffer),
                    span: span.forward(curr_start_span + 2),  // 2 for `f"`
                }]);
            }
            FormatStringParseState::Value => {
                return Err(ParseError::eoe(
                    span.forward(curr_start_span + 2),  // 2 for `f"`
                    ExpectedToken::SpecificTokens(vec![TokenKind::Operator(OpToken::ClosingCurlyBrace)]),
                ));
            }
        }

    }

    Ok(Token {
        kind: TokenKind::FormattedString(buffer),
        span,
    })
}

enum FormatStringParseState {
    String, Value,
}

fn set_span_of_formatted_string(mut tokens: Vec<Token>, span: Span) -> Vec<Token> {

    for token in tokens.iter_mut() {
        token.span = span.forward(token.span.index);
    }

    tokens
}

fn set_span_of_formatted_string_err(mut error: ParseError, span: Span) -> ParseError {
    error.span = span.forward(error.span.index);

    error
}