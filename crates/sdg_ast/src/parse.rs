use crate::err::{ExpectedToken, ParseError};
use crate::expr::{parse_expr, Expr};
use crate::span::Span;
use crate::token::{Token, TokenKind, TokenList};

pub fn split_tokens(tokens: &Vec<Token>, delim: TokenKind) -> Vec<(Vec<Token>, Span)> {
    if tokens.is_empty() {
        return vec![];
    }

    let mut result = vec![];
    let mut curr = vec![];
    let mut span = tokens[0].span;

    for token in tokens.iter() {
        if token.kind == delim {
            result.push((curr, span));
            curr = vec![];
            span = token.span;
        } else {
            curr.push(token.clone());
        }
    }

    if curr.len() > 0 {
        result.push((curr, span));
    }

    result
}

// `elements` is that of `TokenKind::List(_, elements)`
pub fn split_list_by_comma(elements: &Vec<Token>) -> Result<Vec<Expr>, ParseError> {
    let elements = split_tokens(elements, TokenKind::comma()).into_iter().map(
        |(tokens, span)| {
            let mut tokens = TokenList::from_vec(tokens.to_vec(), span);
            parse_expr_exhaustive(&mut tokens)
        }
        );

    let mut elements_buffer = Vec::with_capacity(elements.len());

    for e in elements.into_iter() {
        elements_buffer.push(e?);
    }

    Ok(elements_buffer)
}

pub fn parse_expr_exhaustive(tokens: &mut TokenList) -> Result<Expr, ParseError> {
    match parse_expr(tokens, 0) {
        Ok(e) if tokens.is_eof() => Ok(e),
        Ok(_) => {
            let Token { kind, span } = tokens.step().expect("Internal Compiler Error 3B4CDE2C457");

            Err(ParseError::tok(kind.clone(), *span, ExpectedToken::Nothing))
        }
        Err(e) => Err(e),
    }
}
