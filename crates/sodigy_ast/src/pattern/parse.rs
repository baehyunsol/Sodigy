use super::{PatField, Pattern, PatternKind};
use crate::{IdentWithSpan, Token, TokenKind};
use crate::error::{AstError, AstErrorKind};
use crate::parse::{parse_type_def};
use crate::session::AstSession;
use crate::tokens::Tokens;
use crate::utils::try_into_char;
use crate::warn::AstWarning;
use smallvec::SmallVec;
use sodigy_error::{ErrorContext, ExpectedToken, SodigyError};
use sodigy_intern::InternSession;
use sodigy_lex::QuoteKind;
use sodigy_parse::{Delim, Punct};
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

// TODO: tell the users about operator precedence in patterns
// for now, there's no precedence at all: it just reads from left to right
pub(crate) fn parse_pattern(
    tokens: &mut Tokens,
    session: &mut AstSession,
) -> Result<Pattern, ()> {
    let mut pat = parse_pattern_value(tokens, session)?;

    match tokens.step() {
        Some(Token {
            kind: TokenKind::Punct(punct),
            span,
        }) => {
            let punct_span = *span;

            match punct {
                Punct::At => match pat.try_into_binding() {
                    Some(id) => {
                        let mut rhs = parse_pattern(tokens, session)?;

                        if let Some(binding) = &rhs.bind {
                            session.push_warning(AstWarning::multiple_bindings_on_one_pattern(id, *binding));
                        }

                        rhs.set_bind(id);
                        Ok(rhs)
                    },
                    None => {
                        session.push_error(AstError::expected_binding_got_pattern(pat));
                        return Err(());
                    },
                },
                Punct::Colon => {
                    let ty = parse_type_def(
                        tokens,
                        session,
                        Some(ErrorContext::ParsingTypeInPattern),
                        punct_span,
                    )?;
                    pat.set_ty(ty);

                    Ok(pat)
                },
                p @ (Punct::DotDot
                | Punct::InclusiveRange) => {
                    let p = *p;

                    if tokens.is_curr_token_pattern() {
                        let rhs = parse_pattern(tokens, session)?;
                        let span = pat.span.merge(rhs.span);

                        Ok(Pattern {
                            kind: PatternKind::Range {
                                from: Some(Box::new(pat)),
                                to: Some(Box::new(rhs)),
                                inclusive: matches!(p, Punct::InclusiveRange),
                            },
                            span,
                            bind: None,
                            ty: None,
                        })
                    }

                    else {
                        let span = pat.span.merge(punct_span);

                        Ok(Pattern {
                            kind: PatternKind::Range {
                                from: Some(Box::new(pat)),
                                to: None,
                                inclusive: matches!(p, Punct::InclusiveRange),
                            },
                            span,
                            bind: None,
                            ty: None,
                        })
                    }
                },
                Punct::Or => {
                    let rhs = parse_pattern(tokens, session)?;
                    let span = pat.span.merge(rhs.span);

                    Ok(Pattern {
                        kind: PatternKind::Or(
                            Box::new(pat),
                            Box::new(rhs),
                        ),
                        span,
                        bind: None,
                        ty: None,
                    })
                },
                _ => {
                    tokens.backward().unwrap();

                    Ok(pat)
                },
            }
        },
        other => {
            if other.is_some() {
                tokens.backward().unwrap();
            }

            Ok(pat)
        },
    }
}

// a pattern without operators (`@`, `|`, `..`, )
fn parse_pattern_value(
    tokens: &mut Tokens,
    session: &mut AstSession,
) -> Result<Pattern, ()> {
    let result = match tokens.step() {
        Some(t @ Token {
            kind: TokenKind::Punct(punct),
            span,
        }) => {
            let punct_span = *span;

            match *punct {
                Punct::Dollar => match tokens.expect_ident() {
                    Ok(id) => Pattern {
                        kind: PatternKind::Binding(id.id()),
                        span: punct_span.merge(*id.span()),
                        bind: Some(id),
                        ty: None,
                    },
                    Err(mut e) => {
                        session.push_error(e.set_err_context(
                            ErrorContext::ParsingPattern
                        ).to_owned());
                        return Err(());
                    },
                },
                p @ (Punct::DotDot
                | Punct::InclusiveRange) => {  // prefixed dotdot operator
                    let is_inclusive = p == Punct::InclusiveRange;

                    // there are cases where `parse_pattern` fails but it's not an error
                    // in those cases, `tokens` and `session` have to be restored
                    tokens.take_snapshot();
                    session.take_snapshot();

                    match parse_pattern(tokens, session) {
                        Ok(rhs) => {
                            tokens.pop_snapshot().unwrap();
                            session.pop_snapshot().unwrap();

                            let span = punct_span.merge(rhs.span);

                            Pattern {
                                kind: PatternKind::Range {
                                    from: None,
                                    to: Some(Box::new(rhs)),
                                    inclusive: is_inclusive,
                                },
                                span,
                                bind: None,
                                ty: None,
                            }
                        },
                        Err(_) if !is_inclusive => {
                            // revert the last `parse_pattern` call
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
                        kind: PatternKind::Number {
                            num: n,
                            is_negative: true,
                        },
                        span: punct_span.merge(span),
                        bind: None,
                        ty: None,
                    },
                    Err(mut e) => {
                        session.push_error(e.set_err_context(
                            ErrorContext::ParsingPattern
                        ).to_owned());
                        return Err(());
                    },
                },
                _ => {
                    session.push_error(AstError::unexpected_token(
                        t.clone(),
                        ExpectedToken::pattern(),
                    ).set_err_context(
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
                    ).set_err_context(
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
                        session.push_error(e.set_err_context(
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
                                                session.push_error(e.set_err_context(
                                                    ErrorContext::ParsingPattern
                                                ).to_owned());
                                                return Err(());
                                            }

                                            let pattern = parse_pattern(&mut group_tokens, session)?;

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
                                            session.push_error(e.set_err_context(
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
                                        session.push_error(e.set_err_context(
                                            ErrorContext::ParsingPattern
                                        ).to_owned());
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
            kind: PatternKind::Number {
                num: *n,
                is_negative: false,
            },
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

                let mut intern_session = InternSession::new();
                let content = intern_session.unintern_string(*content).unwrap();

                match try_into_char(content) {
                    Ok(c) => Pattern {
                        kind: PatternKind::Char(c),
                        span: *span,
                        bind: None,
                        ty: None,
                    },
                    Err(e) => {
                        session.push_error(
                            e.into_ast_error(*span).set_err_context(
                                ErrorContext::ParsingPattern
                            ).to_owned()
                        );
                        return Err(());
                    },
                }
            },
            QuoteKind::Double => {
                session.push_error(AstError::todo("string patterns", *span));
                return Err(());
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
                tokens.span_end().unwrap_or(SpanRange::dummy(8)),
                ExpectedToken::pattern(),
            ));
            return Err(());
        },
    };

    Ok(result)
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
            parse_pattern(tokens, session)?
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
                session.push_error(AstError::unexpected_token(
                    tokens.peek().unwrap().clone().clone(),
                    ExpectedToken::nothing(),
                ).set_err_context(
                    ErrorContext::ParsingPattern
                ).to_owned());
                return Err(());
            }
        }
    }
}
