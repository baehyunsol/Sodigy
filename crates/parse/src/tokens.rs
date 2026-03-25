use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};
use sodigy_token::{Token, TokenKind};

pub struct Tokens<'t, 's> {
    pub(crate) tokens: &'t [Token],
    pub(crate) cursor: usize,

    // It's used by `Tokens::unexpected_end`.
    // It's the span of the closing delimitor of the group.
    pub(crate) span_end: Span,
    pub(crate) is_whole_file: bool,

    // In `sodigy_parse`, `Tokens` act like a session.
    pub (crate) intermediate_dir: &'s String,
}

impl<'t, 's> Tokens<'t, 's> {
    pub fn new(
        tokens: &'t [Token],
        span_end: Span,
        is_whole_file: bool,
        intermediate_dir: &'s String,
    ) -> Tokens<'t, 's> {
        Tokens {
            tokens,
            cursor: 0,
            span_end,
            is_whole_file,
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

    pub fn pop_name_and_span(&mut self, allow_wildcard: bool) -> Result<(InternedString, Span), Vec<Error>> {
        match self.peek() {
            Some(Token { kind: TokenKind::Wildcard, span }) => {
                if allow_wildcard {
                    let span = span.clone();
                    self.cursor += 1;
                    Ok((intern_string(b"_", "").unwrap(), span))
                } else {
                    Err(vec![Error {
                        kind: ErrorKind::WildcardNotAllowed,
                        spans: span.simple_error(),
                        note: None,
                    }])
                }
            },
            Some(Token { kind: TokenKind::Ident(id), span }) => {
                let (id, span) = (*id, span.clone());  // bypass the borrow-checker
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

    pub fn unexpected_end(&self, expected: ErrorToken) -> Error {
        let kind = if self.is_whole_file {
            ErrorKind::UnexpectedEof { expected }
        } else {
            ErrorKind::UnexpectedEog { expected }
        };

        Error {
            kind,
            spans: self.span_end.simple_error(),
            note: None,
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

    pub fn peek_prev(&self) -> Option<&Token> {
        match self.cursor {
            0 => None,
            _ => self.tokens.get(self.cursor - 1),
        }
    }

    /// It doesn't care about the cursor!
    pub fn last(&self) -> Option<&Token> {
        self.tokens.last()
    }

    /// It doesn't care about the cursor!
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn enumerate_forward(&self) -> impl Iterator<Item=(usize, &Token)> {
        EnumerateForward::new(self)
    }
}

struct EnumerateForward<'t> {
    tokens: &'t [Token],
    cursor: usize,
}

impl EnumerateForward<'_> {
    pub fn new<'t>(tokens: &Tokens<'t, '_>) -> EnumerateForward<'t> {
        EnumerateForward {
            tokens: tokens.tokens,
            cursor: tokens.cursor,
        }
    }
}

impl<'t> Iterator for EnumerateForward<'t> {
    type Item = (usize, &'t Token);

    fn next(&mut self) -> Option<(usize, &'t Token)> {
        self.cursor += 1;

        match self.tokens.get(self.cursor - 1) {
            Some(token) => Some((self.cursor - 1, token)),
            None => None,
        }
    }
}
