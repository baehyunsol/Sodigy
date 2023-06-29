use crate::err::ParseError;
use crate::expr::{Expr, parse_expr};
use crate::token::{OpToken, Token, TokenKind, TokenList};

pub fn split_tokens(tokens: &Vec<Box<Token>>, delim: TokenKind) -> Vec<Vec<Box<Token>>> {
    let mut result = vec![];
    let mut curr = vec![];

    for token in tokens.iter() {

        if token.kind == delim {
            result.push(curr);
            curr = vec![];
        }

        else {
            curr.push(token.clone());
        }

    }

    if curr.len() > 0 {
        result.push(curr);
    }

    result
}

// `elements` is that of `TokenKind::List(_, elements)`
pub fn split_list_by_comma(elements: &Vec<Box<Token>>) -> Result<Vec<Box<Expr>>, ParseError> {
    let elements = split_tokens(
        elements,
        TokenKind::Operator(OpToken::Comma)
    ).into_iter().map(
        |tokens| parse_expr(&mut TokenList::from_vec_box_token(tokens), 0)
    );

    let mut elements_buffer = Vec::with_capacity(elements.len());

    for e in elements.into_iter() {
        elements_buffer.push(Box::new(e?));
    }

    Ok(elements_buffer)
}