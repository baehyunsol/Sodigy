use crate::err::ParseError;
use crate::parse::split_list_by_comma;
use crate::span::Span;
use crate::token::{Delimiter, Token, TokenKind, TokenList};
use crate::value::{Value, ValueKind};

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
            _ => Err(ParseError::tok(TokenKind::List(*delim, elements.clone()), *span))
        },
        Some(Token { span, kind }) => Err(ParseError::tok(kind.clone(), *span)),

        // upper layer should handle the span
        None => Err(ParseError::eoe(Span::dummy()))
    }

}