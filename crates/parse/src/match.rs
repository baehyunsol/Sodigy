use crate::{Expr, FullPattern, Tokens};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_span::Span;
use sodigy_token::{Delim, Keyword, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Match {
    pub keyword_span: Span,
    pub value: Box<Expr>,
    pub branches: Vec<MatchBranch>,
}

#[derive(Clone, Debug)]
pub struct MatchBranch {
    pub pattern: FullPattern,
    pub cond: Option<Expr>,
    pub value: Expr,
}

impl<'t> Tokens<'t> {
    pub fn parse_match_expr(&mut self) -> Result<Match, Vec<Error>> {
        let keyword = self.match_and_pop(TokenKind::Keyword(Keyword::Match))?;
        let value = self.parse_expr()?;

        let Token {
            kind: TokenKind::Group { tokens, .. },
            span,
        } = self.match_and_pop(TokenKind::Group { delim: Delim::Brace, tokens: vec![] })? else { unreachable!() };
        let mut branch_tokens = Tokens::new(tokens, span.end());
        let branches = branch_tokens.parse_match_branches()?;

        Ok(Match {
            keyword_span: keyword.span,
            value: Box::new(value),
            branches,
        })
    }

    pub fn parse_match_branches(&mut self) -> Result<Vec<MatchBranch>, Vec<Error>> {
        let mut branches = vec![];

        loop {
            let pattern = self.parse_full_pattern()?;

            let cond = match self.peek() {
                Some(Token { kind: TokenKind::Keyword(Keyword::If), .. }) => {
                    self.cursor += 1;
                    let cond = self.parse_expr()?;
                    self.match_and_pop(TokenKind::Punct(Punct::Arrow))?;
                    Some(cond)
                },
                Some(Token { kind: TokenKind::Punct(Punct::Arrow), .. }) => {
                    self.cursor += 1;
                    None
                },
                Some(_) => None,
                None => {
                    return Err(vec![self.unexpected_end(ErrorToken::Punct(Punct::Arrow))]);
                },
            };

            let value = self.parse_expr()?;
            branches.push(MatchBranch {
                pattern,
                cond,
                value,
            });

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
                    break;
                },
                (Some(t), _) => {
                    return Err(vec![Error {
                        kind: ErrorKind::UnexpectedToken {
                            expected: ErrorToken::Punct(Punct::Comma),
                            got: (&t.kind).into(),
                        },
                        span: t.span,
                        ..Error::default()
                    }]);
                },
                (None, _) => {
                    break;
                },
            }
        }

        Ok(branches)
    }
}
