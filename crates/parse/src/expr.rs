use crate::{Block, If, Tokens};
use sodigy_error::{Error, ErrorKind};
use sodigy_keyword::Keyword;
use sodigy_number::InternedNumber;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{ErrorToken, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub enum Expr {
    Identifier {
        id: InternedString,
        span: Span,
    },
    Number {
        n: InternedNumber,
        span: Span,
    },
    If(If),
    Block(Block),
}

impl<'t> Tokens<'t> {
    pub fn parse_expr(&mut self) -> Result<Expr, Vec<Error>> {
        self.pratt_parse(0)
    }

    fn pratt_parse(
        &mut self,
        min_bp: u32,
    ) -> Result<Expr, Vec<Error>> {
        let mut lhs = match self.peek() {
            Some(Token { kind: TokenKind::Identifier(id), span }) => {
                let (id, span) = (*id, *span);
                self.cursor += 1;
                Expr::Identifier { id, span }
            },
            Some(Token { kind: TokenKind::Keyword(Keyword::If), .. }) => Expr::If(self.parse_if_expr()?),
            Some(t) => panic!("TODO: {t:?}"),
            None => {
                return Err(vec![self.unexpected_end(ErrorToken::Expr)]);
            },
        };

        loop {
            match self.peek() {
                Some(Token {
                    kind: TokenKind::Punct(p),
                    span,
                }) => {
                    let punct = *p;
                    let punct_span = *span;
                    panic!("TODO: {punct:?}");
                },
                None => {
                    return Ok(lhs);
                },
                t => panic!("TODO: {t:?}"),
            }
        }
    }

    pub fn parse_comma_separated_expr(&mut self, consume_all: bool) -> Result<Vec<Expr>, Vec<Error>> {
        let mut exprs = vec![];

        if self.peek().is_none() {
            return Ok(exprs);
        }

        loop {
            exprs.push(self.parse_expr()?);

            match self.peek2() {
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
                                expected: ErrorToken::Punct(Punct::Comma),
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
