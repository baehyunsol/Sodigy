use crate::ParseSession;
use sodigy_error::Error;
use sodigy_keyword::Keyword;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Delim, Token, TokenKind};

#[derive(Debug)]
pub struct Func {
    name: InternedString,
    name_span: Span,
    args: Vec<Arg>,
}

#[derive(Debug)]
pub struct Arg {
    name: InternedString,
    name_span: Span,
    r#type: Expr,
}

impl<'t> ParseSession<'t> {
    // KEYWORD_FUNC IDENTIFIER FUNC_ARGS PUNCT_COLON TY_EXPR PUNCT_EQ EXPR PUNCT_SEMICOLON
    pub(crate) fn parse_func(&mut self) -> Result<Func, ()> {
        self.match_and_step(TokenKind::Keyword(Keyword::Func))?;
        let name = self.match_and_step(TokenKind::Identifier(InternedString::dummy()))?;
        let (name, name_span) = match name {
            Token {
                kind: TokenKind::Identifier(name),
                span,
            } => (*name, *span),
            _ => unreachable!(),
        };

        let arg_tokens = self.match_and_step(TokenKind::Group { delim: Delim::Parenthesis, tokens: vec![] })?.unwrap_tokens();
        let args = match parse_args(&arg_tokens) {
            Ok(args) => args,
            Err(e) => {
                self.errors.push(e);
                return Err(());
            },
        };

        Ok(Func {
            name,
            name_span,
            args,
        })
    }
}

fn parse_args(tokens: &[Token]) -> Result<Vec<Arg>, Error> {
    let mut i = 0;

    match tokens.get(i) {
        Some(Token { kind: TokenKind::Identifier(id), span }) => {},
    }
}
