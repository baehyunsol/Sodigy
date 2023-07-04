use super::{Decorator, FuncDef, Stmt, StmtKind, Use, use_case_to_tokens};
use crate::err::{ExpectedToken, ParseError};
use crate::expr::parse_expr;
use crate::module::ModulePath;
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
        // one `use` may generate multiple `Stmt`s, but the return type doesn't allow that
        // so it may modify `tokens` to add `use` cases it found
        match parse_use(tokens) {
            Ok(mut cases) => {
                assert!(cases.len() > 0, "Internal Compiler Error FF61AD7");

                while cases.len() > 1 {
                    tokens.append(use_case_to_tokens(
                        cases.pop().expect("Internal Compiler Error 4151602"),
                    ));
                }

                Ok(Stmt {
                    kind: StmtKind::Use(cases[0].clone()),
                    span: curr_span,
                })
            }
            Err(e) => {
                return Err(e.set_span_of_eof(curr_span));
            }
        }

    } else if tokens.consume(TokenKind::Operator(OpToken::At)) {
        let name = match tokens.step_identifier_strict() {
            Ok(id) => id,
            Err(e) => {
                return Err(e.set_span_of_eof(curr_span));
            }
        };

        let (args, no_args) = match tokens.step_func_args() {
            Some(Ok(args)) => (args, false),
            Some(Err(e)) => {
                return Err(e.set_span_of_eof(curr_span));
            }
            None => (vec![], true)
        };

        Ok(Stmt {
            kind: StmtKind::Decorator(Decorator {
                name,
                args,
                no_args,
            }),
            span: curr_span,
        })
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

/*
 * `use A.B;` -> `use A.B as B;`
 * `use A.B.C;` -> `use A.B.C as C;`
 * `use A.B, C.D;` -> `use A.B; use C.D;`
 * `use {A.B, C.D};` -> `use A.B; use C.D;`
 * `use A.{B, C, D};` -> `use A.B; use A.C; use A.D;`
 * `use A.B, C, D;` -> `use A.B; use C; use D;`
 * `use A.{B as C, D as E};` -> `use A.B as C; use A.D as E;`
 * `use A.{B, C} as D;` -> Invalid
 */
pub fn parse_use(tokens: &mut TokenList) -> Result<Vec<Use>, ParseError> {
    todo!()
}
