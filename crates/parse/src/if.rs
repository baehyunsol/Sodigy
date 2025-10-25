use crate::{Expr, FullPattern, Tokens};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_span::{RenderableSpan, Span};
use sodigy_token::{Delim, Keyword, Punct, Token, TokenKind};

// If there's an `else if` branch,
// that goes into `false_value`, recursively.
#[derive(Clone, Debug)]
pub struct If {
    // If it's `if let`, `if_span` is a merged span of `if` and `let`.
    pub if_span: Span,

    pub cond: Box<Expr>,
    pub pattern: Option<FullPattern>,  // `if let Some((x, _)) = foo() { x + 1 }`

    // If it's `else if`, the span of `else` is stored here,
    // and the span of `if` is stored in `false_value`'s span.
    pub else_span: Span,

    pub true_value: Box<Expr>,
    pub false_value: Box<Expr>,
}

impl<'t> Tokens<'t> {
    pub fn parse_if_expr(&mut self) -> Result<If, Vec<Error>> {
        let mut pattern = None;

        let (if_span, cond) = match self.peek2() {
            // if let PATTERN = COND
            (
                Some(Token { kind: TokenKind::Keyword(Keyword::If), span: span1 }),
                Some(Token { kind: TokenKind::Keyword(Keyword::Let), span: span2 }),
            ) => {
                let span = span1.merge(*span2);
                self.cursor += 2;
                pattern = Some(self.parse_full_pattern()?);
                self.match_and_pop(TokenKind::Punct(Punct::Assign))?;
                (span, self.parse_expr()?)
            },
            // if COND
            (
                Some(Token { kind: TokenKind::Keyword(Keyword::If), span: span1 }),
                Some(_),
            ) => {
                let span1 = *span1;
                self.cursor += 1;
                (span1, self.parse_expr()?)
            },
            (Some(t1), _) => {
                return Err(vec![Error {
                    kind: ErrorKind::UnexpectedToken {
                        expected: ErrorToken::Keyword(Keyword::If),
                        got: (&t1.kind).into(),
                    },
                    spans: t1.span.simple_error(),
                    ..Error::default()
                }]);
            },
            (None, _) => {
                return Err(vec![self.unexpected_end(ErrorToken::Keyword(Keyword::If))]);
            },
        };
        let cond = Box::new(cond);

        let Token {
            kind: TokenKind::Group {
                tokens: true_value_tokens,
                ..
            },
            span: true_value_span,
        } = self.match_and_pop(TokenKind::Group { delim: Delim::Brace, tokens: vec![] })? else { unreachable!() };
        let mut true_value_tokens = Tokens::new(true_value_tokens, true_value_span.end());
        let true_value = Box::new(Expr::block_or_expr(true_value_tokens.parse_block(false /* top-level */, *true_value_span)?));

        let (else_span, false_value) = match self.peek2() {
            (
                Some(Token { kind: TokenKind::Keyword(Keyword::Else), span: span1 }),
                Some(Token { kind: TokenKind::Keyword(Keyword::If), .. }),
            ) => {
                let span1 = *span1;
                self.cursor += 1;
                (span1, Box::new(Expr::If(self.parse_if_expr()?)))
            },
            (
                Some(Token { kind: TokenKind::Keyword(Keyword::Else), span: span1 }),
                Some(Token { kind: TokenKind::Group { delim: Delim::Brace, tokens: false_value_tokens }, span: span2 }),
            ) => {
                let span1 = *span1;
                let span2 = *span2;
                let mut false_value_tokens = Tokens::new(false_value_tokens, span2.end());
                let false_value = Expr::block_or_expr(false_value_tokens.parse_block(false /* top-level */, span2)?);
                self.cursor += 2;
                (span1, Box::new(false_value))
            },
            (
                Some(Token { kind: TokenKind::Keyword(Keyword::Else), .. }),
                Some(t2),
            ) => {
                return Err(vec![Error {
                    kind: ErrorKind::UnexpectedToken {
                        expected: ErrorToken::Block,
                        got: (&t2.kind).into(),
                    },
                    spans: t2.span.simple_error(),
                    ..Error::default()
                }]);
            },
            (
                Some(Token { kind: TokenKind::Keyword(Keyword::Else), .. }),
                None,
            ) => {
                return Err(vec![self.unexpected_end(ErrorToken::Block)]);
            },
            (Some(t1), _) => {
                return Err(vec![Error {
                    kind: ErrorKind::UnexpectedToken {
                        expected: ErrorToken::Keyword(Keyword::Else),
                        got: (&t1.kind).into(),
                    },
                    spans: vec![
                        RenderableSpan {
                            span: t1.span,
                            auxiliary: false,
                            note: None,
                        },
                        RenderableSpan {
                            span: if_span,
                            auxiliary: true,
                            note: Some(String::from("This `if` expression doesn't have a matching `else` expression.")),
                        },
                    ],
                    ..Error::default()
                }]);
            },
            (None, _) => {
                return Err(vec![self.unexpected_end(ErrorToken::Keyword(Keyword::Else))]);
            },
        };

        Ok(If {
            if_span,
            cond,
            pattern,
            else_span,
            true_value,
            false_value,
        })
    }
}
