use crate::err::ParseError;
use crate::expr::Expr;
use crate::session::InternedString;
use crate::span::Span;
use crate::token::{OpToken, TokenKind, TokenList};

// TODO: where should this module belong?

pub struct ArgDef {
    name: InternedString,
    type_: Box<Expr>,   // TODO: better implementation?
}

// NAME ':' TYPE
pub fn parse_arg_def(tokens: &mut TokenList) -> Result<ArgDef, ParseError> {
    assert!(!tokens.is_eof(), "Internal Compiler Error 7109BBF");

    let name = match tokens.step() {
        Some(token) if token.is_identifier() => token.unwrap_identifier(),
        Some(token) => {
            return Err(ParseError::tok(
                token.kind.clone(), token.span,
                vec![TokenKind::Identifier(InternedString::dummy())]
            ));
        }
        None => unreachable!("Internal Compiler Error 53A2FA7")
    };

    tokens.consume_token_or_error(TokenKind::Operator(OpToken::Colon))?;

    let type_ = match tokens.step_type() {
        Some(t) => Box::new(t?),
        None => {
            return Err(ParseError::eoe(Span::dummy()));
        }
    };

    Ok(ArgDef { name, type_ })
}