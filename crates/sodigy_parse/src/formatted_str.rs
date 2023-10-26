use crate::{err::ParseError, from_tokens, ParseSession, TokenTree};
use sodigy_lex::{lex, lex_flex, LexError, LexSession};
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
                                let span_start = span_start.offset(f_string_start_index as i32);

                                lex_flex!(&curr_buf, 0, span_start, lex_session)?;

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

                                let tokens = tmp_parse_session.get_tokens().to_vec();

                                if tokens.is_empty() {
                                    parse_session.push_error(ParseError::empty_f_string(
                                        span_start.extend(span_start.offset(i as i32))
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
                                curr_buf = vec![];
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

    Ok(result)
}

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
