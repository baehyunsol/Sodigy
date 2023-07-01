use super::Stmt;
use crate::err::ParseError;
use crate::expr::parse_expr;
use crate::session::InternedString;
use crate::token::{Keyword, OpToken, TokenKind, TokenList};

pub fn parse_stmts(tokens: &mut TokenList) -> Result<Vec<Stmt>, ParseError> {
    let mut result = vec![];

    while !tokens.is_eof() {
        result.push(parse_stmt(tokens)?);
    }

    Ok(result)
}

pub fn parse_stmt(tokens: &mut TokenList) -> Result<Stmt, ParseError> {
    assert!(!tokens.is_eof(), "Internal Compiler Error FB4375E");

    let curr_span = tokens.get_curr_span().expect("Internal Compiler Error E22AC92");

    if tokens.consume(TokenKind::Keyword(Keyword::Use)) {
        todo!()
    }

    else if tokens.consume(TokenKind::Keyword(Keyword::Def)) {
        let name = match tokens.step() {
            Some(token) if token.is_identifier() => token.unwrap_identifier(),
            Some(token) => {
                return Err(ParseError::tok(
                    token.kind.clone(), token.span,
                    vec![TokenKind::Identifier(InternedString::dummy())]
                ));
            }
            None => {
                return Err(ParseError::eoe(curr_span));
            }
        };

        let args = match tokens.step_func_def_args() {
            Some(Ok(args)) => {
                Some(args)
            }
            Some(Err(e)) => { return Err(e); }
            None => {
                None
            }
        };

        tokens.consume_token_or_error(TokenKind::Operator(OpToken::Assign)).map_err(|e| e.set_span_of_eof(curr_span))?;

        let expr = parse_expr(tokens, 0)?;

        tokens.consume_token_or_error(TokenKind::Operator(OpToken::SemiColon)).map_err(|e| e.set_span_of_eof(curr_span))?;

        todo!()
    }

    else {
        let top_token = tokens.step().expect("Internal Compiler Error 54831A5");

        Err(ParseError::tok(
            top_token.kind.clone(), top_token.span,
            vec![TokenKind::Keyword(Keyword::Use), TokenKind::Keyword(Keyword::Def)]
        ))
    }

}