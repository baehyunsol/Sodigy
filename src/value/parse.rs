use crate::err::{ExpectedToken, ParseError};
use crate::expr::parse_expr;
use crate::parse::split_list_by_comma;
use crate::span::Span;
use crate::stmt::parse_arg_def;
use crate::token::{Delimiter, OpToken, Token, TokenKind, TokenList};
use crate::value::ValueKind;

pub fn parse_value(tokens: &mut TokenList) -> Result<ValueKind, ParseError> {
    match tokens.step() {
        Some(Token {
            span,
            kind: TokenKind::Number(n),
        }) => {
            if n.is_integer() {
                Ok(ValueKind::Integer(n.into()))
            } else {
                Ok(ValueKind::Real(n.clone()))
            }
        }
        Some(Token {
            span,
            kind: TokenKind::String(ind),
        }) => Ok(ValueKind::String(*ind)),
        Some(Token {
            span,
            kind: TokenKind::Identifier(ind),
        }) => Ok(ValueKind::Identifier(*ind)),
        Some(Token {
            span,
            kind: TokenKind::Operator(OpToken::BackSlash)
        }) => {
            // reset lifetime of `span`, so that borrowck doesn't stop me
            let span = *span;

            match tokens.step() {
                Some(Token { kind: TokenKind::List(Delimiter::Brace, elements), .. }) => match parse_lambda_def(
                    &mut TokenList::from_vec_box_token(elements.to_vec())
                ) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(e),
                },
                Some(Token { kind, span }) => Err(ParseError::tok(
                    kind.clone(), *span,
                    ExpectedToken::SpecificTokens(vec![
                        TokenKind::List(Delimiter::Brace, vec![])
                    ])
                )),
                None => Err(ParseError::eoe(
                    span, ExpectedToken::SpecificTokens(vec![
                        TokenKind::List(Delimiter::Brace, vec![])
                    ])
                )),
            }
        },
        Some(Token {
            span,
            kind: TokenKind::List(delim, elements),
        }) => match delim {
            Delimiter::Bracket => Ok(ValueKind::List(split_list_by_comma(elements)?)),
            Delimiter::Brace => {
                parse_block_expr(&mut TokenList::from_vec_box_token(elements.to_vec()))
                    .map_err(|e| e.set_span_of_eof(*span))
            }
            Delimiter::Parenthesis => unreachable!("Internal Compiler Error 2C73648"), // This must be handled by `parse_expr`
        },
        Some(Token { span, kind }) => Err(ParseError::tok(
            kind.clone(),
            *span,
            ExpectedToken::AnyExpression,
        )),

        None => Err(ParseError::eoe(Span::dummy(), ExpectedToken::AnyExpression)),
    }
}

pub fn parse_block_expr(block_tokens: &mut TokenList) -> Result<ValueKind, ParseError> {
    if block_tokens.is_eof() {
        Err(ParseError::eoe_msg(
            Span::dummy(),
            ExpectedToken::AnyExpression,
            "A block cannot be empty!".to_string(),
        ))
    } else if block_tokens.ends_with(TokenKind::semi_colon()) {
        Err(ParseError::eoe_msg(
            block_tokens
                .last_token()
                .expect("Internal Compiler Error B13FA79")
                .span,
            ExpectedToken::AnyExpression,
            "An expression must come at the end of a block".to_string(),
        ))
    } else {
        let first_span = block_tokens
            .peek_curr_span()
            .expect("Internal Compiler Error 64B8455");
        let defs_count =
            block_tokens.count_tokens_non_recursive(TokenKind::semi_colon());

        let mut defs = Vec::with_capacity(defs_count);

        for _ in 0..defs_count {
            let curr_span = block_tokens
                .peek_curr_span()
                .expect("Internal Compiler Error F299389");

            // TODO: allow pattern matchings for assignments
            let name = match block_tokens.step_identifier_strict() {
                Ok(id) => id,
                Err(e) => {
                    assert!(!e.is_eoe(), "Internal Compiler Error 275EFCB");
                    return Err(e);
                }
            };

            block_tokens
                .consume_token_or_error(TokenKind::assign())
                .map_err(|e| e.set_span_of_eof(curr_span))?;

            let expr = parse_expr(block_tokens, 0).map_err(|e| e.set_span_of_eof(curr_span))?;

            block_tokens
                .consume_token_or_error(TokenKind::semi_colon())
                .map_err(|e| e.set_span_of_eof(curr_span))?;

            defs.push((name, Box::new(expr)));
        }

        let value =
            Box::new(parse_expr(block_tokens, 0).map_err(|e| e.set_span_of_eof(first_span))?);

        if let Some(Token { kind, span }) = block_tokens.step() {
            Err(ParseError::tok(kind.clone(), *span, ExpectedToken::Nothing))
        } else {
            Ok(ValueKind::Block { defs, value })
        }
    }
}

// TODO: `parse_block_expr` and `parse_lambda_def` are very similar
fn parse_lambda_def(tokens: &mut TokenList) -> Result<ValueKind, ParseError> {
    if tokens.is_eof() {
        Err(ParseError::eoe_msg(
            Span::dummy(),
            ExpectedToken::AnyExpression,
            "A definition of a lambda function cannot be empty!".to_string(),
        ))
    } else if tokens.ends_with(TokenKind::comma()) {
        Err(ParseError::tok_msg(
            TokenKind::comma(),
            tokens
                .last_token()
                .expect("Internal Compiler Error C929E72")
                .span,
            ExpectedToken::Nothing,
            "Trailing commas in lambda definition is not allowed!".to_string(),
        ))
    } else {
        let first_span = tokens
            .peek_curr_span()
            .expect("Internal Compiler Error 245BA3F");
        let args_count =
            tokens.count_tokens_non_recursive(TokenKind::comma());

        let mut args = Vec::with_capacity(args_count);

        for _ in 0..args_count {
            let curr_span = tokens
                .peek_curr_span()
                .expect("Internal Compiler Error F299389");

            args.push(Box::new(parse_arg_def(tokens).map_err(|e| e.set_span_of_eof(curr_span))?));

            tokens
                .consume_token_or_error(TokenKind::comma())
                .map_err(|e| e.set_span_of_eof(curr_span))?;
        }

        let value =
            Box::new(parse_expr(tokens, 0).map_err(|e| e.set_span_of_eof(first_span))?);

        Ok(ValueKind::Lambda(args, value))
    }
}