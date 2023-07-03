use super::{FuncDef, Stmt, StmtKind};
use crate::err::{ExpectedToken, ParseError};
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

    let curr_span = tokens
        .get_curr_span()
        .expect("Internal Compiler Error E22AC92");

    if tokens.consume(TokenKind::Keyword(Keyword::Use)) {
        todo!()
    } else if tokens.consume(TokenKind::Operator(OpToken::At)) {
        todo!()
    } else if tokens.consume(TokenKind::Keyword(Keyword::Def)) {
        let name = match tokens.step_identifier_strict() {
            Ok(id) => id,
            Err(e) => {
                return Err(e.set_span_of_eof(curr_span));
            }
        };

        let (args, is_const) = match tokens.step_func_def_args() {
            Some(Ok(args)) => (args, false),
            Some(Err(e)) => {
                return Err(e);
            }
            None => (vec![], true),
        };

        tokens
            .consume_token_or_error(TokenKind::Operator(OpToken::Colon))
            .map_err(|e| e.set_span_of_eof(curr_span))?;

        let ret_type = match tokens.step_type() {
            Some(Ok(t)) => t,
            Some(Err(e)) => {
                return Err(e);
            }
            None => {
                return Err(ParseError::eoe_msg(
                    curr_span,
                    ExpectedToken::AnyExpression,
                    "You must provide the return type of this definition!".to_string(),
                ));
            }
        };

        tokens
            .consume_token_or_error(TokenKind::Operator(OpToken::Assign))
            .map_err(|e| e.set_span_of_eof(curr_span))?;

        let ret_val = parse_expr(tokens, 0)?;

        tokens
            .consume_token_or_error(TokenKind::Operator(OpToken::SemiColon))
            .map_err(|e| e.set_span_of_eof(curr_span))?;

        Ok(Stmt {
            kind: StmtKind::Def(FuncDef {
                name,
                args,
                is_const,
                ret_type,
                ret_val,
            }),
            span: curr_span,
        })
    } else {
        let top_token = tokens.step().expect("Internal Compiler Error 54831A5");

        Err(ParseError::tok(
            top_token.kind.clone(),
            top_token.span,
            ExpectedToken::SpecificTokens(vec![
                TokenKind::Keyword(Keyword::Use),
                TokenKind::Keyword(Keyword::Def),
                TokenKind::Operator(OpToken::At),
            ]),
        ))
    }
}
