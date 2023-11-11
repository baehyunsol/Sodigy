use crate::{ParseError, from_tokens, ParseSession, TokenTree};
use crate::warn::ParseWarning;
use sodigy_lex::{lex, LexError, LexSession};
use sodigy_span::SpanPoint;

mod fmt;

#[derive(Clone)]
pub enum FormattedStringElement {
    Value(Vec<TokenTree>),
    Literal(String),
}

enum ParseState {
    Value(usize),  // number of `{`s
    Literal,
}

pub fn parse_str(
    s: &[u8],  // content of string (excluding `f` and `"`s)
    span_start: SpanPoint,  // span of `s[0]`
    lex_session: &mut LexSession,
    parse_session: &mut ParseSession,
) -> Result<Vec<FormattedStringElement>, ()> {
    let mut curr_state = ParseState::Literal;
    let mut curr_buf = vec![];
    let mut result = vec![];
    let mut f_string_start_index = 0;

    for (i, c) in s.iter().enumerate() {
        match &mut curr_state {
            ParseState::Literal => {
                if *c == b'{' {
                    match String::from_utf8(curr_buf) {
                        Ok(s) => {
                            result.push(FormattedStringElement::Literal(s));
                            curr_buf = vec![];
                        },
                        Err(_) => {
                            lex_session.push_error(LexError::invalid_utf8(
                                span_start.into_range()
                            ));
                            return Err(());
                        },
                    }

                    curr_state = ParseState::Value(1);
                    f_string_start_index = i + 1;
                }

                else {
                    curr_buf.push(*c);
                }
            },
            ParseState::Value(n) => {
                curr_buf.push(*c);

                if *c == b'{' {
                    *n += 1;
                }

                else if *c == b'}' {
                    *n -= 1;

                    if *n == 0 {
                        // pop last '}'
                        curr_buf.pop().unwrap();

                        match check_fstring_type(&curr_buf) {
                            t @ (FstringType::Normal | FstringType::NestedNested) => {
                                lex_session.flush_tokens();
                                let f_string_start_span = span_start.offset(
                                    f_string_start_index as i32
                                    + (t == FstringType::NestedNested) as i32
                                );

                                lex(pill_braces(&curr_buf, t), 0, f_string_start_span, lex_session)?;

                                let tokens = lex_session.get_tokens().to_vec();

                                // it has to `parse_session.flush_tokens` before calling `from_tokens`
                                // but it has to store the current `parse_session.tokens` at some location -> we cannot lose that!
                                let mut tmp_parse_session = ParseSession::from_lex_session(&lex_session);

                                if let Err(()) = from_tokens(&tokens, &mut tmp_parse_session, lex_session) {
                                    for err in tmp_parse_session.get_errors() {
                                        parse_session.push_error(err.clone());
                                    }

                                    return Err(());
                                }

                                for warning in tmp_parse_session.get_warnings() {
                                    parse_session.push_warning(warning.clone());
                                }

                                let tokens = tmp_parse_session.get_tokens().to_vec();

                                if tokens.is_empty() {
                                    parse_session.push_error(ParseError::empty_f_string(
                                        f_string_start_span.offset(-1).extend(span_start.offset(i as i32 + 1))
                                    ));

                                    return Err(());
                                }

                                if let FstringType::NestedNested = t {
                                    result.push(FormattedStringElement::Literal(String::from("{")));
                                    result.push(FormattedStringElement::Value(tokens));
                                    result.push(FormattedStringElement::Literal(String::from("}")));
                                }

                                else {
                                    result.push(FormattedStringElement::Value(tokens));
                                }

                                curr_state = ParseState::Literal;
                                curr_buf.clear();
                            },
                            FstringType::Nested => {
                                result.push(FormattedStringElement::Literal(String::from_utf8(curr_buf).unwrap()));
                                curr_state = ParseState::Literal;
                                curr_buf = vec![];
                            },
                        }
                    }
                }
            },
        }
    }

    match curr_state {
        ParseState::Literal => {
            if result.is_empty() {
                parse_session.push_warning(ParseWarning::nothing_to_eval_in_f_string(span_start.offset(-1).into_range()));
            }

            match String::from_utf8(curr_buf) {
                Ok(s) => {
                    result.push(FormattedStringElement::Literal(s));
                },
                Err(_) => {
                    lex_session.push_error(LexError::invalid_utf8(
                        span_start.into_range()
                    ));
                    return Err(());
                },
            }
        },
        ParseState::Value(_) => {
            parse_session.push_warning(ParseWarning::unmatched_curly_brace(span_start.offset(f_string_start_index as i32 - 1).into_range()));

            result.push(FormattedStringElement::Literal(String::from("{")));

            match String::from_utf8(curr_buf) {
                Ok(s) => {
                    result.push(FormattedStringElement::Literal(s));
                },
                Err(_) => {
                    lex_session.push_error(LexError::invalid_utf8(
                        span_start.into_range()
                    ));
                    return Err(());
                },
            }
        },
    }

    Ok(result)
}

fn pill_braces(buf: &[u8], f_string_type: FstringType) -> &[u8] {
    match f_string_type {
        FstringType::Normal => buf,
        FstringType::NestedNested => &buf[1..(buf.len() - 1)],
        _ => unreachable!(),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum FstringType {
    Normal,        // f"{3 + 4}" -> "7"
    Nested,        // f"{{3 + 4}}" -> "{3 + 4}"
    NestedNested,  // f"{{{3 + 4}}}" -> "{7}"
}

fn check_fstring_type(buf: &[u8]) -> FstringType {
    if buf.first() == Some(&b'{') && buf.last() == Some(&b'}') {
        if buf.get(1) == Some(&b'{') && buf.get(buf.len() - 2) == Some(&b'}') {
            FstringType::NestedNested
        }

        else {
            FstringType::Nested
        }
    }

    else {
        FstringType::Normal
    }
}
