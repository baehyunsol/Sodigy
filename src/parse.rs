use crate::err::{ExpectedToken, ParseError};
use crate::expr::{parse_expr, Expr};
use crate::token::{Token, TokenKind, TokenList};

pub fn split_tokens(tokens: &Vec<Token>, delim: TokenKind) -> Vec<Vec<Token>> {
    let mut result = vec![];
    let mut curr = vec![];

    for token in tokens.iter() {
        if token.kind == delim {
            result.push(curr);
            curr = vec![];
        } else {
            curr.push(token.clone());
        }
    }

    if curr.len() > 0 {
        result.push(curr);
    }

    result
}

// `elements` is that of `TokenKind::List(_, elements)`
pub fn split_list_by_comma(elements: &Vec<Token>) -> Result<Vec<Expr>, ParseError> {
    let elements = split_tokens(elements, TokenKind::comma())
        .into_iter()
        .map(|tokens| {
            let mut tokens = TokenList::from_vec(tokens.to_vec());

            parse_expr_exhaustive(&mut tokens)
        });

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
            let Token { kind, span } = tokens.step().expect("Internal Compiler Error 72A64FD");

            Err(ParseError::tok(kind.clone(), *span, ExpectedToken::Nothing))
        }
        Err(e) => Err(e),
    }
}
