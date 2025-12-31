use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Token, TokenKind};

pub struct Tokens<'t, 's> {
    pub(crate) tokens: &'t [Token],
    pub(crate) cursor: usize,

    // It's used by `Tokens::unexpected_end`.
    // It can be Span::Eof or the span of the closing delimitor of the group.
    pub(crate) span_end: Span,

    // In `sodigy_parse`, `Tokens` act like a session.
    pub (crate) intermediate_dir: &'s String,
}

impl<'t, 's> Tokens<'t, 's> {
    pub fn new(tokens: &'t [Token], span_end: Span, intermediate_dir: &'s String) -> Tokens<'t, 's> {
        Tokens {
            tokens,
            cursor: 0,
            span_end,
            intermediate_dir,
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
                    spans: t.span.simple_error(),
                    note: None,
                }]);
            },
            None => {
                return Err(vec![self.unexpected_end((&token).into())]);
            },
        }
    }

    pub fn pop_name_and_span(&mut self) -> Result<(InternedString, Span), Vec<Error>> {
        match self.peek() {
            Some(Token { kind: TokenKind::Ident(id), span }) => {
                let (id, span) = (*id, *span);  // bypass the borrow-checker
                self.cursor += 1;
                Ok((id, span))
            },
            Some(t) => Err(vec![Error {
                kind: ErrorKind::UnexpectedToken {
                    expected: ErrorToken::Ident,
                    got: (&t.kind).into(),
                },
                spans: t.span.simple_error(),
                note: None,
            }]),
            None => Err(vec![self.unexpected_end(ErrorToken::Ident)]),
        }
    }

    pub fn unexpected_end(&self, expected_token: ErrorToken) -> Error {
        match self.span_end {
            Span::Lib | Span::Std | Span::Eof(_) | Span::File(_) | Span::None => Error {
                kind: ErrorKind::UnexpectedEof {
                    expected: expected_token,
                },
                spans: self.span_end.simple_error(),
                note: None,
            },
            Span::Range { .. } | Span::Derived { .. } => Error {
                kind: ErrorKind::UnexpectedEog {
                    expected: expected_token,
                },
                spans: self.span_end.simple_error(),
                note: None,
            },
            Span::Prelude(_) => unreachable!(),
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

    pub fn peek3(&self) -> (Option<&Token>, Option<&Token>, Option<&Token>) {
        (
            self.tokens.get(self.cursor),
            self.tokens.get(self.cursor + 1),
            self.tokens.get(self.cursor + 2),
        )
    }

    pub fn peek4(&self) -> (Option<&Token>, Option<&Token>, Option<&Token>, Option<&Token>) {
        (
            self.tokens.get(self.cursor),
            self.tokens.get(self.cursor + 1),
            self.tokens.get(self.cursor + 2),
            self.tokens.get(self.cursor + 3),
        )
    }

    /// It doesn't care about the cursor!
    pub fn last(&self) -> Option<&Token> {
        self.tokens.last()
    }

    /// It doesn't care about the cursor!
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}
