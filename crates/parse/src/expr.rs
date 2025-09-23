use crate::Tokens;
use sodigy_error::{Error, ErrorKind};
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{ErrorToken, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Expr {
    kind: ExprKind,
    span: Span,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Identifier(InternedString),
}

impl<'t> Tokens<'t> {
    pub fn parse_expr(&mut self) -> Result<Expr, Vec<Error>> {
        match self.tokens.get(self.cursor) {
            Some(Token { kind: TokenKind::Identifier(id), span }) => Ok(Expr {
                kind: ExprKind::Identifier(*id),
                span: *span,
            }),
            Some(t) => panic!("TODO: {t:?}"),
            None => Err(vec![self.unexpected_end(ErrorToken::Expr)]),
        }
    }

    pub fn parse_comma_separated_expr(&mut self, consume_all: bool) -> Result<Vec<Expr>, Vec<Error>> {
        let mut exprs = vec![];

        if self.peek().is_none() {
            return Ok(exprs);
        }

        loop {
            exprs.push(self.parse_expr()?);

            match (self.tokens.get(self.cursor), self.tokens.get(self.cursor + 1)) {
                (
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }),
                    Some(_),
                ) => {
                    self.cursor += 1;
                },
                (
                    Some(Token { kind: TokenKind::Punct(Punct::Comma), .. }),
                    None,
                ) => {
                    self.cursor += 1;
                    break;
                },
                (None, _) => {
                    break;
                },
                (Some(t), _) => {
                    if consume_all {
                        return Err(vec![Error {
                            kind: ErrorKind::UnexpectedToken {
                                expected: ErrorToken::Comma,
                                got: (&t.kind).into(),
                            },
                            span: t.span,
                        }]);
                    }

                    else {
                        break;
                    }
                },
            }
        }

        Ok(exprs)
    }
}
