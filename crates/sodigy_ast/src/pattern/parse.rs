use super::{PatField, Pattern, PatternKind};
use crate::{IdentWithSpan, Token, TokenKind};
use crate::error::{AstError, AstErrorKind};
use crate::parse::{parse_type_def};
use crate::session::AstSession;
use crate::tokens::Tokens;
use crate::utils::{try_into_char, IntoCharErr};
use crate::warn::AstWarning;
use smallvec::SmallVec;
use sodigy_error::{ErrorContext, ExpectedToken, SodigyError};
use sodigy_lex::QuoteKind;
use sodigy_parse::{Delim, Punct};
use sodigy_session::SodigySession;
use sodigy_span::SpanRange;

// operators
// PAT `:` TY
// BIND `@` PAT
// PAT `|` PAT
// `(` PAT? `)`
//     -> an empty tuple is also a valid pattern
// PAT? `..` PAT?

// groups
// NAME `{` FIELDS `}`
// NAME? `(` PATS `)`
// `[` PATS `]`

// units
// BIND = `$` IDENT
// NAME = IDENT (`.` IDENT)*
// NUMERIC
// CHAR
// DOTDOT

// Precedence
//  1. `..`, `..~`
//  2. `@`
//  3. `|`
//  4. `:`

// There's a function for each level of precedence.
// `parse_pattern_full` parses an entire pattern, including `@`s, `|`s and `:`s.
// `parse_pattern_no_annotation` parses everything but type annotations.
// `parse_pattern_with_binding` parses a pattern with `@`s.
// `parse_pattern_value` parses a pattern without `@`s, `|`s and `:`s.

// A type annotation after `:` is an expression, and `..` and `|` are valid operators. But that doesn't make any problem
// because the operator precedence separates patterns and expressions.

// `:` level precedence
pub fn parse_pattern_full(
    tokens: &mut Tokens,
    session: &mut AstSession,
) -> Result<Pattern, ()> {
    let mut lhs = parse_pattern_no_annotation(tokens, session)?;

    match tokens.peek() {
        Some(Token {
            kind: TokenKind::Punct(Punct::Colon),
            span,
        }) => {
            let punct_span = *span;
            tokens.step().unwrap();

            let ty = parse_type_def(
                tokens,
                session,
                punct_span,
            )?;
            lhs.set_ty(ty);

            Ok(lhs)
        },
        _ => Ok(lhs),
    }
}

// `|` level precedence
fn parse_pattern_no_annotation(
    tokens: &mut Tokens,
    session: &mut AstSession,
) -> Result<Pattern, ()> {
    let mut lhs = parse_pattern_with_binding(tokens, session)?;

    while let Some(Token {
        kind: TokenKind::Punct(Punct::Or),
        span,
    }) = tokens.peek() {
        let or_span = *span;
        tokens.step().unwrap();

        let rhs = parse_pattern_with_binding(tokens, session)?;

        lhs = Pattern {
            kind: PatternKind::Or(Box::new(lhs), Box::new(rhs)),
            span: or_span,
            bind: None,
            ty: None,
        };
    }

    Ok(lhs)
}

// `@` level precedence
fn parse_pattern_with_binding(
    tokens: &mut Tokens,
    session: &mut AstSession,
) -> Result<Pattern, ()> {
    match parse_pattern_value(tokens, session) {
        ref p @ Ok(Pattern {
            kind: PatternKind::Binding(binding),
            span,
            ..
        }) => match tokens.peek() {
            Some(Token {
                kind: TokenKind::Punct(Punct::At),
                ..
            }) => {
                let binding = IdentWithSpan::new(binding, span);

                tokens.step().unwrap();

                let mut rhs = parse_pattern_value(tokens, session)?;

                if let Some(old_binding) = &rhs.bind {
                    session.push_warning(AstWarning::multiple_bindings_on_one_pattern(binding, *old_binding));
                }

                rhs.set_bind(binding);
                Ok(rhs)
            },
            _ => p.clone(),
        },
        res => res,
    }
}

// `..` level precedence
fn parse_pattern_value(
    tokens: &mut Tokens,
    session: &mut AstSession,
) -> Result<Pattern, ()> {
    let mut lhs = match tokens.step() {
        Some(t @ Token {
            kind: TokenKind::Punct(punct),
            span,
        }) => {
            let punct_span = *span;

            match *punct {
                Punct::Dollar => match tokens.expect_ident() {
                    Ok(mut id) => {
                        id.set_span(punct_span.merge(*id.span()));

                        Pattern {
                            kind: PatternKind::Binding(id.id()),
                            span: *id.span(),
                            bind: Some(id),
                            ty: None,
                        }
                    },
                    Err(mut e) => {
                        session.push_error(e.set_error_context(
                            ErrorContext::ParsingPattern
                        ).to_owned());
                        return Err(());
                    },
                },
                p @ (Punct::DotDot
                | Punct::InclusiveRange) => {  // prefixed dotdot operator
                    let is_inclusive = p == Punct::InclusiveRange;

                    // there are cases where `parse_pattern_value` fails but it's not an error
                    // in those cases, `tokens` and `session` have to be restored
                    tokens.take_snapshot();
                    session.take_snapshot();

                    match parse_pattern_value(tokens, session) {
                        Ok(rhs) => {
                            tokens.pop_snapshot().unwrap();
                            session.pop_snapshot().unwrap();
                            let is_string = rhs.is_string();

                            rhs.assert_no_type_and_no_binding(session)?;

                            Pattern {
                                kind: PatternKind::Range {
                                    from: None,
                                    to: Some(Box::new(rhs)),
                                    inclusive: is_inclusive,
                                    is_string,
                                },
                                span: punct_span,
                                bind: None,
                                ty: None,
                            }
                        },
                        Err(_) if !is_inclusive => {
                            // revert the last `parse_pattern_value` call
                            tokens.restore_to_last_snapshot();
                            session.restore_to_last_snapshot();

                            Pattern {
                                kind: PatternKind::Shorthand,
                                span: punct_span,
                                bind: None,
                                ty: None,
                            }
                        },
                        Err(_) => {
                            tokens.pop_snapshot().unwrap();
                            session.pop_snapshot().unwrap();

                            return Err(());
                        },
                    }
                },
                Punct::Sub => match tokens.expect_number() {
                    Ok((n, span)) => Pattern {
                        kind: PatternKind::Number(n.neg()),
                        span: punct_span.merge(span),
                        bind: None,
                        ty: None,
                    },
                    Err(mut e) => {
                        session.push_error(e.set_error_context(
                            ErrorContext::ParsingPattern
                        ).to_owned());
                        return Err(());
                    },
                },
                _ => {
                    session.push_error(AstError::unexpected_token(
                        t.clone(),
                        ExpectedToken::pattern(),
                    ).set_error_context(
                        ErrorContext::ParsingPattern
                    ).to_owned());
                    return Err(());
                },
            }
        },
        Some(t @ Token {
            kind: TokenKind::Group { delim, tokens, prefix: b'\0' },
            span,
        }) => {
            let group_span = *span;
            let delim = *delim;
            let mut tokens = tokens.to_vec();
            let mut tokens = Tokens::from_vec(&mut tokens);
            tokens.set_span_end(group_span.last_char());

            match delim {
                Delim::Paren => {  // a pattern inside parenthesis, or a tuple
                    let (patterns, has_trailing_comma) = parse_comma_separated_patterns(
                        &mut tokens,
                        session,
                        /* must_consume_all_tokens */ true,
                    )?;

                    if patterns.len() == 1 && !has_trailing_comma {
                        patterns[0].clone()
                    }

                    else {
                        Pattern {
                            kind: PatternKind::Tuple(patterns),
                            span: group_span,
                            bind: None,
                            ty: None,
                        }
                    }
                },
                Delim::Bracket => {  // list
                    let (patterns, _) = parse_comma_separated_patterns(
                        &mut tokens,
                        session,
                        /* must_consume_all_tokens */ true,
                    )?;

                    Pattern {
                        kind: PatternKind::List(patterns),
                        span: group_span,
                        bind: None,
                        ty: None,
                    }
                },
                Delim::Brace => {  // err
                    session.push_error(AstError::unexpected_token(
                        t.clone(),
                        ExpectedToken::pattern(),
                    ).set_error_context(
                        ErrorContext::ParsingPattern
                    ).to_owned());
                    return Err(());
                },
            }
        },
        Some(Token {
            kind: TokenKind::Identifier(id),
            span,
        }) => {
            let mut name_span = *span;
            let mut names = vec![
                IdentWithSpan::new(*id, *span),
            ];

            while tokens.is_curr_token(TokenKind::dot()) {
                tokens.step().unwrap();

                names.push(match tokens.expect_ident() {
                    Ok(id) => {
                        name_span = name_span.merge(*id.span());

                        id
                    },
                    Err(mut e) => {
                        session.push_error(e.set_error_context(
                            ErrorContext::ParsingPattern
                        ).to_owned());
                        return Err(());
                    },
                });
            }

            if names.len() == 1 && names[0].id().is_underbar() {
                return Ok(Pattern {
                    kind: PatternKind::Wildcard,
                    span: *names[0].span(),
                    bind: None,
                    ty: None,
                });
            }

            match tokens.step() {
                Some(Token {
                    kind: TokenKind::Group { delim, tokens: group_tokens, prefix: b'\0' },
                    span,
                }) => {
                    let span = *span;
                    let mut group_tokens = group_tokens.to_vec();
                    let mut group_tokens = Tokens::from_vec(&mut group_tokens);
                    group_tokens.set_span_end(span.last_char());

                    match delim {
                        Delim::Paren => {
                            let (fields, _) = parse_comma_separated_patterns(
                                &mut group_tokens,
                                session,
                                /* must_consume_all_tokens */ true,
                            )?;

                            Pattern {
                                kind: PatternKind::TupleStruct {
                                    name: names,
                                    fields,
                                },
                                span: name_span.merge(span),
                                bind: None,
                                ty: None,
                            }
                        },
                        Delim::Brace => {
                            let mut pat_fields = vec![];
                            let mut shorthand_spans = SmallVec::<[SpanRange; 1]>::new();

                            loop {
                                if group_tokens.is_finished() {
                                    break;
                                }

                                if group_tokens.is_curr_token(TokenKind::Punct(Punct::DotDot)) {
                                    shorthand_spans.push(group_tokens.step().unwrap().span);
                                }

                                else {
                                    match group_tokens.expect_ident() {
                                        Ok(id) => {
                                            if let Err(mut e) = group_tokens.consume(TokenKind::colon()) {
                                                session.push_error(e.set_error_context(
                                                    ErrorContext::ParsingPattern
                                                ).to_owned());
                                                return Err(());
                                            }

                                            let pattern = parse_pattern_full(&mut group_tokens, session)?;

                                            pat_fields.push(PatField {
                                                name: id,
                                                pattern,
                                            });
                                        },
                                        Err(AstError {
                                            kind: AstErrorKind::UnexpectedEnd(_),
                                            ..
                                        }) => {},
                                        Err(mut e) => {
                                            session.push_error(e.set_error_context(
                                                ErrorContext::ParsingPattern
                                            ).to_owned());
                                            return Err(());
                                        },
                                    }
                                }

                                match group_tokens.consume(TokenKind::comma()) {
                                    Ok(_) => {
                                        continue;
                                    },
                                    Err(AstError {
                                        kind: AstErrorKind::UnexpectedEnd(_),
                                        ..
                                    }) => {
                                        break;
                                    },
                                    Err(mut e) => {
                                        e.set_error_context(ErrorContext::ParsingPattern);

                                        if let Some(Token {
                                            kind: TokenKind::Punct(Punct::At),
                                            ..
                                        }) = group_tokens.peek() {
                                            e.set_message(String::from("To bind a name to a pattern, the name must come before the pattern, not after it."));
                                        }

                                        session.push_error(e);
                                        return Err(());
                                    },
                                }
                            }

                            if shorthand_spans.len() > 1 {
                                session.push_error(AstError::multiple_shorthands_in_one_pattern(shorthand_spans));
                                return Err(());
                            }

                            Pattern {
                                kind: PatternKind::Struct {
                                    struct_name: names,
                                    has_shorthand: shorthand_spans.len() > 0,
                                    fields: pat_fields,
                                },
                                span: name_span.merge(span),
                                bind: None,
                                ty: None,
                            }
                        },
                        Delim::Bracket => {
                            tokens.backward().unwrap();

                            let pttk = if names.len() == 1 {
                                PatternKind::Identifier(names[0].id())
                            } else {
                                PatternKind::Path(names)
                            };
        
                            Pattern {
                                kind: pttk,
                                span: name_span,
                                bind: None,
                                ty: None,
                            }
                        },
                    }
                },
                etc => {
                    if etc.is_some() {
                        tokens.backward().unwrap();
                    }

                    let pttk = if names.len() == 1 {
                        PatternKind::Identifier(names[0].id())
                    } else {
                        PatternKind::Path(names)
                    };

                    Pattern {
                        kind: pttk,
                        span: name_span,
                        bind: None,
                        ty: None,
                    }
                },
            }
        },
        Some(Token {
            kind: TokenKind::Number(n),
            span,
        }) => Pattern {
            kind: PatternKind::Number(*n),
            span: *span,
            bind: None,
            ty: None,
        },
        Some(Token {
            kind: TokenKind::String {
                kind: q_kind,
                content,
                is_binary,
            },
            span,
        }) => match *q_kind {
            QuoteKind::Single => {
                if *is_binary {
                    session.push_error(AstError::binary_char(*span));
                    return Err(());
                }

                else if let Some((length, bytes)) = content.try_unwrap_short_string() {
                    match try_into_char(&bytes[0..(length as usize)]) {
                        Ok(c) => Pattern {
                            kind: PatternKind::Char(c),
                            span: *span,
                            bind: None,
                            ty: None,
                        },
                        Err(e) => {
                            session.push_error(
                                e.into_ast_error(*span).set_error_context(
                                    ErrorContext::ParsingPattern
                                ).to_owned()
                            );
                            return Err(());
                        },
                    }
                }

                else {
                    session.push_error(
                        IntoCharErr::TooLong.into_ast_error(*span).set_error_context(
                            ErrorContext::ParsingPattern
                        ).to_owned()
                    );
                    return Err(());
                }
            },
            QuoteKind::Double => Pattern {
                kind: PatternKind::String {
                    content: *content,
                    is_binary: *is_binary,
                },
                span: *span,
                bind: None,
                ty: None,
            },
        },
        Some(token) => {
            session.push_error(AstError::unexpected_token(
                token.clone(),
                ExpectedToken::pattern(),
            ));
            return Err(());
        },
        None => {
            session.push_error(AstError::unexpected_end(
                tokens.span_end().unwrap_or(SpanRange::dummy()),
                ExpectedToken::pattern(),
            ));
            return Err(());
        },
    };

    // check if the next token is `..`
    // if so, continue parsing
    match tokens.peek() {
        Some(Token {
            kind: TokenKind::Punct(p @ (Punct::DotDot | Punct::InclusiveRange)),
            span,
        }) => {
            let range_span = *span;
            let is_inclusive = matches!(p, Punct::InclusiveRange);

            tokens.step().unwrap();

            // there are cases where `parse_pattern_value` fails but it's not an error
            // in those cases, `tokens` and `session` have to be restored
            tokens.take_snapshot();
            session.take_snapshot();

            match parse_pattern_value(tokens, session) {
                Ok(rhs) => {
                    tokens.pop_snapshot().unwrap();
                    session.pop_snapshot().unwrap();
                    let is_string = rhs.is_string() || lhs.is_string();

                    lhs.assert_no_type_and_no_binding(session)?;
                    rhs.assert_no_type_and_no_binding(session)?;

                    lhs = Pattern {
                        kind: PatternKind::Range {
                            from: Some(Box::new(lhs)),
                            to: Some(Box::new(rhs)),
                            inclusive: is_inclusive,
                            is_string,
                        },
                        span: range_span,
                        bind: None,
                        ty: None,
                    }
                },
                Err(_) => {
                    // revert the last `parse_pattern_value` call
                    tokens.restore_to_last_snapshot();
                    session.restore_to_last_snapshot();
                    let is_string = lhs.is_string();

                    lhs.assert_no_type_and_no_binding(session)?;

                    lhs = Pattern {
                        kind: PatternKind::Range {
                            from: Some(Box::new(lhs)),
                            to: None,
                            inclusive: is_inclusive,
                            is_string,
                        },
                        span: range_span,
                        bind: None,
                        ty: None,
                    }
                },
            }
        },
        _ => {},
    }

    Ok(lhs)
}

type TrailingComma = bool;

fn parse_comma_separated_patterns(
    tokens: &mut Tokens,
    session: &mut AstSession,
    must_consume_all_tokens: bool,
) -> Result<(Vec<Pattern>, TrailingComma), ()> {
    let mut has_trailing_comma = false;
    let mut patterns = vec![];

    loop {
        if tokens.is_finished() {
            return Ok((patterns, has_trailing_comma));
        }

        patterns.push(
            parse_pattern_full(tokens, session)?
        );

        has_trailing_comma = false;

        if tokens.is_curr_token(TokenKind::comma()) {
            tokens.step().unwrap();
            has_trailing_comma = true;
            continue;
        }

        else {
            if tokens.is_finished() || !must_consume_all_tokens {
                return Ok((patterns, has_trailing_comma));
            }

            else {
                let last_token = tokens.peek().unwrap().clone();
                let mut e = AstError::unexpected_token(
                    last_token.clone(),
                    ExpectedToken::nothing(),
                );
                e.set_error_context(ErrorContext::ParsingPattern);

                if let TokenKind::Punct(Punct::At) = &last_token.kind {
                    e.set_message(String::from("To bind a name to a pattern, the name must come before the pattern, not after it."));
                }

                session.push_error(e);
                return Err(());
            }
        }
    }
}
