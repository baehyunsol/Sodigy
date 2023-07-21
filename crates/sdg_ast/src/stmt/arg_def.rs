use crate::err::{ExpectedToken, ParseError};
use crate::expr::Expr;
use crate::session::InternedString;
use crate::span::Span;
use crate::token::{OpToken, TokenKind, TokenList};
use crate::value::BlockDef;

#[derive(Clone)]
pub struct ArgDef {
    pub name: InternedString,

    // if it's None, it has to be inferred later
    pub ty: Option<Expr>,

    // first character of the name
    pub span: Span,
}

// NAME ':' TYPE
pub fn parse_arg_def(tokens: &mut TokenList) -> Result<ArgDef, ParseError> {
    assert!(!tokens.is_eof(), "Internal Compiler Error 7109BBF");
    let span = tokens.peek_curr_span().expect("Internal Compiler Error 266A2FE");

    let name = match tokens.step_identifier_strict() {
        Ok(id) => id,
        Err(e) => {
            assert!(!e.is_eoe(), "Internal Compiler Error 53A2FA7");

            return Err(e);
        }
    };

    let colon_span = tokens.peek_curr_span();

    if tokens.consume(TokenKind::Operator(OpToken::Colon)) {
        let ty = match tokens.step_type() {
            Some(t) => Some(t?),
            None => {
                return Err(ParseError::eoe(colon_span.expect("Internal Compiler Error 2CBC3AD"), ExpectedToken::AnyExpression));
            }
        };

        Ok(ArgDef { name, ty, span })
    }

    else {
        Ok(ArgDef { name, ty: None, span })
    }

}

// TODO: where should this belong?
pub trait GetNameOfArg {
    fn get_name_of_arg(&self) -> InternedString;
}

impl GetNameOfArg for ArgDef {
    fn get_name_of_arg(&self) -> InternedString {
        self.name
    }
}

impl GetNameOfArg for BlockDef {
    fn get_name_of_arg(&self) -> InternedString {
        self.name
    }
}

impl<A: GetNameOfArg> GetNameOfArg for Box<A> {
    fn get_name_of_arg(&self) -> InternedString {
        self.as_ref().get_name_of_arg()
    }
}