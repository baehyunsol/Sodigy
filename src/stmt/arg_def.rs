use crate::err::{ExpectedToken, ParseError};
use crate::expr::Expr;
use crate::session::InternedString;
use crate::span::Span;
use crate::token::{OpToken, TokenKind, TokenList};

#[derive(Clone)]
pub struct ArgDef {
    pub name: InternedString,
    pub ty: Expr,
}

// NAME ':' TYPE
pub fn parse_arg_def(tokens: &mut TokenList) -> Result<ArgDef, ParseError> {
    assert!(!tokens.is_eof(), "Internal Compiler Error 7109BBF");

    let name = match tokens.step_identifier_strict() {
        Ok(id) => id,
        Err(e) => {
            assert!(!e.is_eoe(), "Internal Compiler Error 53A2FA7");

            return Err(e);
        }
    };

    tokens.consume_token_or_error(TokenKind::Operator(OpToken::Colon))?;

    let ty = match tokens.step_type() {
        Some(t) => t?,
        None => {
            return Err(ParseError::eoe(Span::dummy(), ExpectedToken::AnyExpression));
        }
    };

    Ok(ArgDef { name, ty })
}
