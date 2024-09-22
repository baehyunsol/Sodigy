use super::{PatField, Pattern, PatternKind};
use crate::{IdentWithSpan, Token, TokenKind};
use crate::error::{AstError, AstErrorKind};
use crate::parse::{parse_type_def};
use crate::session::AstSession;
use crate::tokens::Tokens;
use crate::utils::{IntoCharError, try_into_char};
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

    if let Some(Token {
        kind: TokenKind::Punct(Punct::Colon),
        span,
    }) = tokens.peek() {
        let punct_span = *span;
        tokens.step().unwrap();

        let ty = parse_type_def(
            tokens,
            session,
            punct_span,
        )?;
        lhs.set_ty(ty);
    }

    let or_pattern_expansion_limit = session.get_or_pattern_expansion_limit();
    let unfolded = match unfold_or_patterns(&lhs, or_pattern_expansion_limit, session) {
        Ok(u) => u,
        Err(_) => {
            session.push_error(AstError::excessive_or_pattern(lhs.span, or_pattern_expansion_limit));
            return Err(());
        },
    };

    if unfolded.len() == 1 {
        Ok(unfolded[0].clone())
    }

    else {
        Ok(Pattern {
            kind: PatternKind::Or(unfolded),
            ..lhs.clone()
        })
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
            kind: PatternKind::OrRaw(Box::new(lhs), Box::new(rhs)),
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

            match punct {
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
                    let is_inclusive = matches!(p, Punct::InclusiveRange);

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
                                            e.push_message(String::from("To bind a name to a pattern, the name must come before the pattern, not after it."));
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
                                    ErrorContext::ParsingPattern,
                                ).to_owned()
                            );
                            return Err(());
                        },
                    }
                }

                else {
                    session.push_error(
                        IntoCharError::TooLong.into_ast_error(*span).set_error_context(
                            ErrorContext::ParsingPattern,
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
                    e.push_message(String::from("To bind a name to a pattern, the name must come before the pattern, not after it."));
                }

                session.push_error(e);
                return Err(());
            }
        }
    }
}

// TODO: This function is very inefficient in 2 ways.
// 1. It has tons of `clone`s, vector allocation and redundant searches.
// 2. It naively unfolds all the patterns, which easily gets very big.
//    For example `((1 | 2), (3 | 4), (5 | 6), (7 | 8))` would be unfolded to 16 patterns.
fn unfold_or_patterns(
    p: &Pattern,
    length_limit: usize,  // it returns Err if it exceeds limit

    // it only collects warnings. errors are collected by its callee
    session: &mut AstSession,
) -> Result<Vec<Pattern>, ()> {
    match &p.kind {
        PatternKind::Identifier(_)
        | PatternKind::Number(_)
        | PatternKind::Char(_)
        | PatternKind::String { .. }
        | PatternKind::Binding(_)
        | PatternKind::Path(_)
        | PatternKind::Wildcard
        | PatternKind::Shorthand => Ok(vec![p.clone()]),
        PatternKind::Range {
            from,
            to,
            inclusive,
            is_string,
        } => {
            let from = match from.as_ref() {
                Some(pattern) => Some(unfold_or_patterns(
                    pattern.as_ref(),
                    length_limit,
                    session,
                )?),
                None => None,
            };
            let to = match to.as_ref() {
                Some(pattern) => Some(unfold_or_patterns(
                    pattern.as_ref(),
                    length_limit,
                    session,
                )?),
                None => None,
            };
            let from_unfolded = from.as_ref().map(|patterns| patterns.len() > 1).unwrap_or(false);
            let to_unfolded = to.as_ref().map(|patterns| patterns.len() > 1).unwrap_or(false);

            if !from_unfolded && !to_unfolded {
                Ok(vec![p.clone()])
            }

            else {
                // converting `Option<Vec<Pattern>>` to `Vec<Option<Pattern>>`
                // it makes the iteration easier
                let from = match from {
                    Some(v) => v.into_iter().map(|x| Some(x)).collect(),
                    None => vec![None],
                };
                let to = match to {
                    Some(v) => v.into_iter().map(|x| Some(x)).collect(),
                    None => vec![None],
                };

                let mut result = Vec::with_capacity(from.len() * to.len());

                if result.capacity() > length_limit {
                    return Err(());
                }

                for f in from.iter() {
                    for t in to.iter() {
                        result.push(Pattern {
                            kind: PatternKind::Range {
                                from: f.as_ref().map(|f| Box::new(f.clone())),
                                to: t.as_ref().map(|t| Box::new(t.clone())),
                                inclusive: *inclusive,
                                is_string: *is_string,
                            },
                            ..p.clone()
                        });
                    }
                }

                Ok(result)
            }
        },
        p_kind @ (PatternKind::Tuple(patterns)
        | PatternKind::List(patterns)) => {
            let is_tuple = matches!(p_kind, PatternKind::Tuple(_));
            let mut has_recursive_or_pattern = false;
            let mut unfolded_patterns = Vec::with_capacity(patterns.len());

            for pattern in patterns.iter() {
                let u = unfold_or_patterns(
                    pattern,
                    length_limit,
                    session,
                )?;

                if u.len() > 1 {
                    has_recursive_or_pattern = true;
                }

                unfolded_patterns.push(u);
            }

            if has_recursive_or_pattern {
                let unfolded_patterns = permutation(
                    &patterns,
                    &unfolded_patterns,
                    length_limit,
                    session,
                )?;

                Ok(unfolded_patterns.into_iter().map(
                    |patterns| Pattern {
                        kind: if is_tuple { PatternKind::Tuple(patterns) } else { PatternKind::List(patterns) },
                        ..p.clone()
                    }
                ).collect())
            }

            else {
                Ok(vec![p.clone()])
            }
        },
        PatternKind::TupleStruct { name, fields: patterns } => {
            let mut has_recursive_or_pattern = false;
            let mut unfolded_patterns = Vec::with_capacity(patterns.len());

            for pattern in patterns.iter() {
                let u = unfold_or_patterns(
                    pattern,
                    length_limit,
                    session,
                )?;

                if u.len() > 1 {
                    has_recursive_or_pattern = true;
                }

                unfolded_patterns.push(u);
            }

            if has_recursive_or_pattern {
                let unfolded_patterns = permutation(
                    &patterns,
                    &unfolded_patterns,
                    length_limit,
                    session,
                )?;

                Ok(unfolded_patterns.into_iter().map(
                    |patterns| Pattern {
                        kind: PatternKind::TupleStruct { name: name.clone(), fields: patterns },
                        ..p.clone()
                    }
                ).collect())
            }

            else {
                Ok(vec![p.clone()])
            }
        },
        PatternKind::Struct { 
            struct_name,
            fields,
            has_shorthand,
         } => {
            let mut has_recursive_or_pattern = false;
            let mut unfolded_patterns = Vec::with_capacity(fields.len());

            for field in fields.iter() {
                let u = unfold_or_patterns(
                    &field.pattern,
                    length_limit,
                    session,
                )?;

                if u.len() > 1 {
                    has_recursive_or_pattern = true;
                }

                unfolded_patterns.push(u);
            }

            if has_recursive_or_pattern {
                let unfolded_patterns = permutation(
                    &fields.iter().map(
                        |PatField { pattern, .. }| pattern.clone()
                    ).collect::<Vec<_>>(),
                    &unfolded_patterns,
                    length_limit,
                    session,
                )?;

                Ok(unfolded_patterns.into_iter().map(
                    |patterns| Pattern {
                        kind: PatternKind::Struct {
                            struct_name: struct_name.clone(),
                            fields: patterns.into_iter().zip(fields.iter()).map(
                                |(unfolded_pattern, field)| PatField {
                                    name: field.name,
                                    pattern: unfolded_pattern,
                                }
                            ).collect(),
                            has_shorthand: *has_shorthand,
                        },
                        ..p.clone()
                    }
                ).collect())
            }

            else {
                Ok(vec![p.clone()])
            }
        },
        PatternKind::OrRaw(left, right) => {
            let mut left = unfold_or_patterns(
                left.as_ref(),
                length_limit,
                session,
            )?;
            let mut right = unfold_or_patterns(
                right.as_ref(),
                length_limit,
                session,
            )?;

            if left.len() + right.len() > length_limit {
                Err(())
            }

            else {
                if let Some(bind) = &p.bind {
                    for ll in left.iter_mut() {
                        if ll.bind.is_some() {
                            session.push_warning(AstWarning::multiple_bindings_on_one_pattern(
                                ll.bind.clone().unwrap(),
                                bind.clone(),
                            ))
                        }

                        ll.bind = Some(bind.clone());
                    }

                    for rr in right.iter_mut() {
                        if rr.bind.is_some() {
                            session.push_warning(AstWarning::multiple_bindings_on_one_pattern(
                                rr.bind.clone().unwrap(),
                                bind.clone(),
                            ))
                        }

                        rr.bind = Some(bind.clone());
                    }
                }

                // TODO: it has to do the same thing on type annotations, but I'm not sure whether
                //       multiple_type_annotation_on_one_pattern is an error or a warning
                //       plus, the type checker will find errors later. what I'm concerned is that

                Ok(vec![
                    left,
                    right,
                ].concat())
            }
        },
        PatternKind::Or(patterns) => {
            Ok(patterns.clone())
        },
    }
}

// TODO: It's too naive
//
// annotated_patterns
// [$x, $y, $z]
//
// patterns_to_permute
// [[1, 2, 3], [4, 5], [6]]
//
// result
// [
//     [$x @ 1, $y @ 4, $z @ 6],
//     [$x @ 2, $y @ 4, $z @ 6],
//     [$x @ 3, $y @ 4, $z @ 6],
//     [$x @ 1, $y @ 5, $z @ 6],
//     [$x @ 2, $y @ 5, $z @ 6],
//     [$x @ 3, $y @ 5, $z @ 6],
// ]
fn permutation(
    // the result has to inherit name bindings and type annotations from the original patterns
    annotated_patterns: &Vec<Pattern>,

    patterns_to_permute: &Vec<Vec<Pattern>>,
    length_limit: usize,  // it returns Err if it exceeds the limit

    // it only collects warnings. errors are collected by someone else
    session: &mut AstSession,
) -> Result<Vec<Vec<Pattern>>, ()> {
    let mut result = vec![vec![]];

    if patterns_to_permute.iter().map(|e| e.len()).product::<usize>() > length_limit {
        return Err(());
    }

    for (index, v) in patterns_to_permute.iter().enumerate() {
        let mut new_result = vec![];
        let curr_annotation = &annotated_patterns[index];

        for x in v.iter() {
            for r in result.iter() {
                let mut new_x = x.clone();
                let mut new_r = r.clone();
                // for `$a @ ($b @ 1 | $c @ 2)`, new_x.bind is $c and curr_annotation.bind is $a

                if curr_annotation.bind.is_some() {
                    if new_x.bind.is_some() {
                        let mut w = AstWarning::multiple_bindings_on_one_pattern(
                            new_x.bind.clone().unwrap(),
                            curr_annotation.bind.clone().unwrap(),
                        );
                        w.push_message(
                            String::from("In this case, the compiler ignores one of the binding and it might lead to a confusion in name collision checking. `($x @ 1 | $x @ 2)` is anti-pattern. Use `$x @ (1 | 2)`"),
                        );
                        session.push_warning(w);
                    }

                    new_x.bind = curr_annotation.bind.clone();
                }

                if curr_annotation.ty.is_some() {
                    new_x.ty = curr_annotation.ty.clone();
                }

                new_r.push(new_x.clone());
                new_result.push(new_r);
            }
        }

        result = new_result;
    }

    Ok(result)
}
