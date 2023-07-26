use super::ValueKind;
use crate::ast::NameOrigin;
use crate::err::{ExpectedToken, ParseError, ParamType};
use crate::expr::parse_expr;
use crate::parse::{parse_expr_exhaustive, split_list_by_comma};
use crate::span::Span;
use crate::stmt::parse_arg_def;
use crate::token::{Delimiter, Keyword, OpToken, Token, TokenKind, TokenList};
use crate::value::BlockDef;
use sdg_uid::UID;
use std::collections::HashSet;

pub fn parse_value(tokens: &mut TokenList) -> Result<ValueKind, ParseError> {
    match tokens.step() {
        Some(Token {
            kind: TokenKind::Number(n),
            ..
        }) => if n.is_integer() {
                Ok(ValueKind::Integer(n.into()))
        } else {
            Ok(ValueKind::Real(n.clone()))
        },
        Some(Token {
            kind: TokenKind::String(buf),
            ..
        }) => Ok(ValueKind::String(buf.to_vec())),
        Some(Token {
            kind: TokenKind::Identifier(ind),
            ..
        }) => Ok(ValueKind::Identifier(*ind, NameOrigin::NotKnownYet)),
        Some(Token {
            span,
            kind: TokenKind::Operator(OpToken::BackSlash),
        }) => {
            // reset lifetime of `span`, so that borrowck doesn't stop me
            let span = *span;

            match tokens.step() {
                Some(Token { kind: TokenKind::List(Delimiter::Brace, elements), .. }) => match parse_lambda_def(
                    &mut TokenList::from_vec(elements.to_vec())
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
            kind: TokenKind::Bytes(b),
            ..
        }) => Ok(ValueKind::Bytes(b.to_vec())),
        Some(Token {
            kind: TokenKind::FormattedString(tokens),
            ..
        }) => {
            // very simple optimization: `f"ABC"` -> `"ABC"`
            if tokens.len() == 1 && tokens[0].len() == 1 && tokens[0][0].kind.is_string() {
                Ok(ValueKind::String(tokens[0][0].kind.unwrap_string().to_vec()))
            }

            else if tokens.is_empty() {
                Ok(ValueKind::String(vec![]))
            }

            else {
                let exprs = tokens.iter().map(
                    |tokens| {
                        let start_span = tokens[0].span;
                        let mut tokens = TokenList::from_vec(tokens.to_vec());

                        parse_expr_exhaustive(&mut tokens).map_err(|e| e.set_span_of_eof(start_span))
                    }
                );
                let mut buffer = Vec::with_capacity(tokens.len());

                for expr in exprs.into_iter() {
                    buffer.push(expr?);
                }

                Ok(ValueKind::Format(buffer))
            }
        },
        Some(Token {
            span,
            kind: TokenKind::List(delim, elements),
        }) => match delim {
            Delimiter::Bracket => Ok(ValueKind::List(
                split_list_by_comma(elements).map_err(
                    |mut e| {
                        e.set_expected_tokens_instead_of_nothing(vec![
                            TokenKind::Operator(OpToken::ClosingSquareBracket),
                            TokenKind::comma(),
                        ]);

                        e
                    }
                )?
            )),
            Delimiter::Brace => {
                parse_block_expr(&mut TokenList::from_vec(elements.to_vec()))
                    .map_err(|e| e.set_span_of_eof(*span))
            }
            Delimiter::Parenthesis => unreachable!("Internal Compiler Error 35C353D8706"), // This must be handled by `parse_expr`
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
            "A block cannot be empty.".to_string(),
        ))
    } else if block_tokens.ends_with(TokenKind::semi_colon()) {
        Err(ParseError::eoe_msg(
            block_tokens
                .last_token()
                .expect("Internal Compiler Error 36756872BD9")
                .span,
            ExpectedToken::AnyExpression,
            "An expression must come at the end of a block.".to_string(),
        ))
    } else {
        let first_span = block_tokens
            .peek_curr_span()
            .expect("Internal Compiler Error 5450834BC6F");
        let defs_count =
            block_tokens.count_tokens_non_recursive(TokenKind::semi_colon());

        let mut defs = Vec::with_capacity(defs_count);
        let mut names = HashSet::with_capacity(defs_count);

        for _ in 0..defs_count {
            // points to `let`
            let curr_span = block_tokens
                .peek_curr_span()
                .expect("Internal Compiler Error 3DB2B1546F6");

            block_tokens
                .consume_token_or_error(vec![TokenKind::Keyword(Keyword::Let)])
                .map_err(|e| e.set_span_of_eof(curr_span))?;

            // points to the first character of the name (or the pattern)
            let name_span = block_tokens.peek_curr_span();

            // TODO: allow pattern matchings for assignments
            // -> Don't change the shape of `BlockDef`,
            //    Just convert a pattern into 1 or more `BlockDef`s in this function
            // -> ex: `Person { age, name } = foo();`
            //    into `_tmp: Person = foo(); age = _tmp.age; name = _tmp.name;`
            let name = block_tokens.step_identifier_strict()?;

            // Now it's guaranteed that it's not None
            let name_span = name_span.expect("Internal Compiler Error A455FA3AFC7");

            if !names.insert(name) {
                return Err(ParseError::multi_def(name, name_span, ParamType::BlockDef));
            }

            // type annotation is optional
            let ty = if block_tokens.consume(TokenKind::colon()) {
                Some(parse_expr(block_tokens, 0).map_err(|e| e.set_span_of_eof(name_span))?)
            } else {
                None
            };

            block_tokens
                .consume_token_or_error(vec![TokenKind::assign()])
                .map_err(|e| e.set_span_of_eof(name_span))?;

            let expr = parse_expr(block_tokens, 0).map_err(|e| e.set_span_of_eof(name_span))?;

            block_tokens
                .consume_token_or_error(vec![TokenKind::semi_colon()])
                .map_err(|e| e.set_span_of_eof(name_span))?;

            defs.push(BlockDef { name, ty, value: expr, span: name_span });
        }

        let value =
            Box::new(parse_expr(block_tokens, 0).map_err(|e| e.set_span_of_eof(first_span))?);

        if let Some(Token { kind, span }) = block_tokens.step() {
            Err(ParseError::tok(
                kind.clone(),
                *span,
                ExpectedToken::SpecificTokens(vec![TokenKind::Operator(OpToken::ClosingCurlyBrace)])
            ))
        } else {
            Ok(ValueKind::Block { defs, value, id: UID::new_block_id() })
        }
    }
}

fn parse_lambda_def(tokens: &mut TokenList) -> Result<ValueKind, ParseError> {
    if tokens.is_eof() {
        Err(ParseError::eoe_msg(
            Span::dummy(),
            ExpectedToken::AnyExpression,
            "A definition of a lambda function cannot be empty.".to_string(),
        ))
    } else if tokens.ends_with(TokenKind::comma()) {
        Err(ParseError::tok_msg(
            TokenKind::comma(),
            tokens
                .last_token()
                .expect("Internal Compiler Error 0A486D27A08")
                .span,
            ExpectedToken::Nothing,
            "Trailing commas in lambda definition is not allowed.".to_string(),
        ))
    } else {
        let first_span = tokens
            .peek_curr_span()
            .expect("Internal Compiler Error 92BC0C79DD6");
        let args_count =
            tokens.count_tokens_non_recursive(TokenKind::comma());

        let mut args = Vec::with_capacity(args_count);
        let mut arg_names = HashSet::with_capacity(args_count);

        for _ in 0..args_count {
            let curr_span = tokens
                .peek_curr_span()
                .expect("Internal Compiler Error EF60F059587");

            let arg = parse_arg_def(tokens).map_err(|e| e.set_span_of_eof(curr_span))?;

            if !arg_names.insert(arg.name) {
                return Err(ParseError::multi_def(arg.name, arg.span, ParamType::LambdaParam));
            }

            args.push(arg);

            tokens
                .consume_token_or_error(vec![TokenKind::comma()])
                .map_err(|e| e.set_span_of_eof(curr_span))?;
        }

        let value = parse_expr(tokens, 0).map_err(|e| e.set_span_of_eof(first_span))?;

        Ok(ValueKind::Lambda(args, Box::new(value)))
    }
}
