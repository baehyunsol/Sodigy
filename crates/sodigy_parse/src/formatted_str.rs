use crate::{ParseError, from_tokens, ParseSession, TokenTree};
use crate::warn::ParseWarning;
use sodigy_lex::{lex, FSTRING_START_MARKER, LexError, LexSession};
use sodigy_span::SpanPoint;

mod endec;
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
    let mut fstring_start_index = 0;
    let mut has_error = false;
    let mut encountered_braces_so_far = 0;

    for (i, c) in s.iter().enumerate() {
        match &mut curr_state {
            ParseState::Literal => {
                if *c == FSTRING_START_MARKER {
                    match String::from_utf8(curr_buf) {
                        Ok(s) => {
                            result.push(FormattedStringElement::Literal(s));
                        },
                        Err(_) => {
                            lex_session.push_error(LexError::invalid_utf8(
                                span_start.into_range()
                            ));

                            has_error = true;
                        },
                    }

                    curr_buf = vec![];
                    curr_state = ParseState::Value(1);
                    fstring_start_index = i + 2 + encountered_braces_so_far;
                    encountered_braces_so_far += 1;
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

                        lex_session.flush_tokens();
                        let fstring_start_span = span_start.offset(
                            fstring_start_index as i32
                        );

                        lex(&curr_buf, 0, fstring_start_span, lex_session)?;

                        let tokens = lex_session.get_tokens().to_vec();

                        // it has to `parse_session.flush_tokens` before calling `from_tokens`
                        // but it has to store the current `parse_session.tokens` at some location -> we cannot lose that!
                        let mut tmp_parse_session = ParseSession::from_lex_session(&lex_session);

                        if let Err(()) = from_tokens(&tokens, &mut tmp_parse_session, lex_session) {
                            for err in tmp_parse_session.get_errors() {
                                parse_session.push_error(err.clone());
                            }

                            has_error = true;
                        }

                        for warning in tmp_parse_session.get_warnings() {
                            parse_session.push_warning(warning.clone());
                        }

                        let tokens = tmp_parse_session.get_tokens().to_vec();

                        if tokens.is_empty() {
                            parse_session.push_error(ParseError::empty_fstring(
                                fstring_start_span.offset(-1).extend(span_start.offset(i as i32 + 1))
                            ));

                            has_error = true;
                        }

                        result.push(FormattedStringElement::Value(tokens));

                        curr_state = ParseState::Literal;
                        curr_buf.clear();
                    }
                }
            }
        }
    }

    match curr_state {
        ParseState::Literal => {
            if result.is_empty() {
                parse_session.push_warning(ParseWarning::nothing_to_eval_in_fstring(span_start.offset(-1).into_range()));
            }

            match String::from_utf8(curr_buf) {
                Ok(s) => {
                    result.push(FormattedStringElement::Literal(s));
                },
                Err(_) => {
                    lex_session.push_error(LexError::invalid_utf8(
                        span_start.into_range()
                    ));

                    has_error = true;
                },
            }
        },

        // lexer guarantees that all the curly braces are terminated
        ParseState::Value(_) => unreachable!(),
    }

    if has_error {
        Err(())
    }

    else {
        Ok(result)
    }
}
