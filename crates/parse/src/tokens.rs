use sodigy_error::{Error, ErrorKind};
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{ErrorToken, Token, TokenKind};

pub struct Tokens<'t> {
    pub(crate) tokens: &'t [Token],
    pub(crate) cursor: usize,

    // It's used by `Tokens::unexpected_end`.
    // It can be Span::Eof or the span of the closing delimitor of the group.
    span_end: Span,
}

impl<'t> Tokens<'t> {
    pub fn new(tokens: &'t [Token], span_end: Span) -> Tokens<'t> {
        Tokens {
            tokens,
            cursor: 0,
            span_end,
        }
    }

    // In this world, every `Result` returns `Vec<Error>` instead of just `Error`.
    // That's because some parsers might return multiple errors,
    // and mixing `Result<_, Error>` and `Result<_, Vec<Error>>` makes the code messy.
    pub fn match_and_pop(&mut self, token: TokenKind) -> Result<&'t Token, Vec<Error>> {
        // If we use `self.peek()` here, the borrow-checker will refuse it.
        match self.tokens.get(self.cursor) {
            Some(t) if t.kind.matches(&token) => {
                self.cursor += 1;
                Ok(t)
            },
            Some(t) => {
                return Err(vec![Error {
                    kind: ErrorKind::UnexpectedToken {
                        expected: (&token).into(),
                        got: (&t.kind).into(),
                    },
                    span: t.span,
                }]);
            },
            None => {
                return Err(vec![self.unexpected_end((&token).into())]);
            },
        }
    }

    pub fn pop_name_and_span(&mut self) -> Result<(InternedString, Span), Vec<Error>> {
        match self.peek() {
            Some(Token { kind: TokenKind::Identifier(id), span }) => {
                let (id, span) = (*id, *span);  // bypass the borrow-checker
                self.cursor += 1;
                Ok((id, span))
            },
            Some(t) => Err(vec![Error {
                kind: ErrorKind::UnexpectedToken {
                    expected: ErrorToken::Identifier,
                    got: (&t.kind).into(),
                },
                span: t.span,
            }]),
            None => Err(vec![self.unexpected_end(ErrorToken::Identifier)]),
        }
    }

    pub fn unexpected_end(&self, expected_token: ErrorToken) -> Error {
        match self.span_end {
            Span::Eof(_) | Span::File(_) | Span::None => Error {
                kind: ErrorKind::UnexpectedEof {
                    expected: expected_token,
                },
                span: self.span_end,
            },
            Span::Range { .. } => Error {
                kind: ErrorKind::UnexpectedEog {
                    expected: expected_token,
                },
                span: self.span_end,
            },
        }
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    pub fn peek2(&self) -> (Option<&Token>, Option<&Token>) {
        (
            self.tokens.get(self.cursor),
            self.tokens.get(self.cursor + 1),
        )
    }
}
