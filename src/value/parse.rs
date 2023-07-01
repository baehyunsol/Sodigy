use crate::err::ParseError;
use crate::expr::parse_expr;
use crate::parse::split_list_by_comma;
use crate::session::InternedString;
use crate::span::Span;
use crate::token::{Delimiter, OpToken, Token, TokenKind, TokenList};
use crate::value::{Value, ValueKind};
use hmath::Ratio;

pub fn parse_value(tokens: &mut TokenList) -> Result<Value, ParseError> {

    match tokens.step() {
        Some(Token { span, kind: TokenKind::Number(n) }) => if n.is_integer() {
            Ok(Value { span: *span, kind: ValueKind::Integer(n.into()) })
        } else {
            Ok(Value { span: *span, kind: ValueKind::Real(n.clone()) })
        },
        Some(Token { span, kind: TokenKind::String(ind) }) => Ok(Value {
            span: *span,
            kind: ValueKind::String(*ind)
        }),
        Some(Token { span, kind: TokenKind::Identifier(ind) }) => Ok(Value {
            span: *span,
            kind: ValueKind::Identifier(*ind)
        }),
        Some(Token { span, kind: TokenKind::List(delim, elements) }) => match delim {
            Delimiter::Bracket => Ok(Value {
                span: *span,
                kind: ValueKind::List(split_list_by_comma(elements)?)
            }),
            Delimiter::Brace => parse_block_expr(
                &mut TokenList::from_vec_box_token(elements.to_vec())
            ).map_err(|e| e.set_span_of_eof(*span)),
            Delimiter::Parenthesis => unreachable!("Internal Compiler Error 2C73648"),  // This must be handled by `parse_expr`
        },
        Some(Token { span, kind }) => Err(ParseError::tok(
            kind.clone(), *span,
            vec![
                TokenKind::Number(Ratio::zero()),  // `render_err` will not show the actual value
                TokenKind::String(InternedString::dummy()),
                TokenKind::Identifier(InternedString::dummy()),
                TokenKind::List(Delimiter::Bracket, vec![]),
                TokenKind::List(Delimiter::Brace, vec![]),
                TokenKind::List(Delimiter::Parenthesis, vec![]),
            ]
        )),

        // upper layer should handle the span
        None => Err(ParseError::eoe(Span::dummy()))
    }

}

pub fn parse_block_expr(block_tokens: &mut TokenList) -> Result<Value, ParseError> {

    if block_tokens.is_eof() {
        Err(ParseError::eoe_msg(Span::dummy(), "A block cannot be empty!".to_string()))
    }

    else if block_tokens.ends_with(TokenKind::Operator(OpToken::SemiColon)) {
        Err(ParseError::eoe_msg(
            block_tokens.last_token().expect("Internal Compiler Error B13FA79").span,
            "An expression must come at the end of a block".to_string()
        ))
    }

    else {
        let first_span = block_tokens.get_curr_span().expect("Internal Compiler Error 64B8455");
        let mut defs_count = block_tokens.count_tokens_non_recursive(TokenKind::Operator(OpToken::SemiColon));

        let mut defs = Vec::with_capacity(defs_count);

        for _ in 0..defs_count {
            let curr_span = block_tokens.get_curr_span().expect("Internal Compiler Error F299389");

            // TODO: allow pattern matchings for assignments
            let name = match block_tokens.step() {
                Some(Token { kind, .. }) if kind.is_identifier() => kind.unwrap_identifier(),
                Some(Token { kind, span }) => {
                    return Err(ParseError::tok(
                        kind.clone(), *span,
                        vec![TokenKind::Identifier(InternedString::dummy())]
                    ));
                }
                None => unreachable!("Interal Compiler Error 275EFCB")
            };

            block_tokens.consume_token_or_error(
                TokenKind::Operator(OpToken::Assign)
            ).map_err(|e| e.set_span_of_eof(curr_span))?;

            let expr = parse_expr(block_tokens, 0).map_err(|e| e.set_span_of_eof(curr_span))?;

            block_tokens.consume_token_or_error(
                TokenKind::Operator(OpToken::SemiColon)
            ).map_err(|e| e.set_span_of_eof(curr_span))?;

            defs.push((name, Box::new(expr)));
        }

        let value = Box::new(parse_expr(block_tokens, 0).map_err(|e| e.set_span_of_eof(first_span))?);

        Ok(Value { kind: ValueKind::Block { defs, value }, span: first_span })
    }
}