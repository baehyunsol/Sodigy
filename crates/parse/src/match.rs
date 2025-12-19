use crate::{Expr, ParsePatternContext, Pattern, Tokens};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_span::Span;
use sodigy_token::{Delim, Keyword, Punct, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Match {
    pub keyword_span: Span,
    pub scrutinee: Box<Expr>,
    pub arms: Vec<MatchArm>,
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub value: Expr,
}

impl<'t, 's> Tokens<'t, 's> {
    pub fn parse_match_expr(&mut self) -> Result<Match, Vec<Error>> {
        let keyword = self.match_and_pop(TokenKind::Keyword(Keyword::Match))?;
        let scrutinee = self.parse_expr()?;

        let Token {
            kind: TokenKind::Group { tokens, .. },
            span,
        } = self.match_and_pop(TokenKind::Group { delim: Delim::Brace, tokens: vec![] })? else { unreachable!() };
        let mut arm_tokens = Tokens::new(tokens, span.end(), &self.intermediate_dir);
        let arms = arm_tokens.parse_match_arms()?;

        Ok(Match {
            keyword_span: keyword.span,
            scrutinee: Box::new(scrutinee),
            arms,
        })
    }

    pub fn parse_match_arms(&mut self) -> Result<Vec<MatchArm>, Vec<Error>> {
        let mut arms = vec![];

        loop {
            let pattern = self.parse_pattern(ParsePatternContext::MatchArm)?;

            let guard = match self.peek() {
                Some(Token { kind: TokenKind::Keyword(Keyword::If), .. }) => {
                    self.cursor += 1;
                    let guard = self.parse_expr()?;
                    self.match_and_pop(TokenKind::Punct(Punct::Arrow))?;
                    Some(guard)
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
            arms.push(MatchArm {
                pattern,
                guard,
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
                        spans: t.span.simple_error(),
                        note: None,
                    }]);
                },
                (None, _) => {
                    break;
                },
            }
        }

        Ok(arms)
    }
}
