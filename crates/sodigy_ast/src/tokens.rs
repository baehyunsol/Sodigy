use crate::IdentWithSpan;
use crate::err::{ExpectedToken, AstError};
use crate::{Token, TokenKind};
use sodigy_intern::InternedNumeric;
use sodigy_keyword::Keyword;
use sodigy_parse::{Delim, Punct};
use sodigy_span::SpanRange;

#[derive(Debug)]
pub struct Tokens<'t> {
    data: &'t mut Vec<Token>,
    cursor: usize,

    // if self.data.is_empty, self.span_end() returns this span
    span_end_: Option<SpanRange>,
    snapshots: Vec<TokensSnapshot>,
}

impl<'t> Tokens<'t> {
    pub fn from_vec(data: &'t mut Vec<Token>) -> Self {
        Tokens {
            data,
            cursor: 0,
            span_end_: None,
            snapshots: vec![],
        }
    }

    pub fn set_span_end(&mut self, span_end: SpanRange) {
        self.span_end_ = Some(span_end);
    }

    pub fn is_finished(&self) -> bool {
        self.data.len() <= self.cursor
    }

    pub fn peek(&self) -> Option<&Token> {
        self.data.get(self.cursor)
    }

    pub fn peek_span(&self) -> Option<SpanRange> {
        self.data.get(self.cursor).map(|t| t.span)
    }

    pub fn step(&mut self) -> Option<&Token> {
        self.cursor += 1;
        self.data.get(self.cursor - 1)
    }

    // returns Err if it cannot go backward
    pub fn backward(&mut self) -> Result<(), ()> {
        if self.cursor == 0 {
            Err(())
        }

        else {
            self.cursor -= 1;
            Ok(())
        }
    }

    pub fn span_end(&self) -> Option<SpanRange> {
        match self.data.last().map(|t| t.span.last_char()) {
            Some(s) => Some(s),
            None => self.span_end_,
        }
    }

    // returns true if it finds a statement
    pub fn march_until_stmt(&mut self) -> bool {
        loop {
            match self.peek() {
                Some(Token { kind, .. }) => match kind {
                    TokenKind::Keyword(k) => match k {
                        Keyword::Def
                        | Keyword::Enum
                        | Keyword::Struct
                        | Keyword::Module
                        | Keyword::Import => {
                            return true;
                        },
                        Keyword::If
                        | Keyword::Else
                        | Keyword::As
                        | Keyword::From
                        | Keyword::Let
                        | Keyword::Match => {
                            self.cursor += 1;
                        },
                    },
                    TokenKind::Punct(Punct::At) => {
                        return true;
                    },
                    TokenKind::DocComment(_) => {
                        return true;
                    },
                    _ => {
                        self.cursor += 1;
                    }
                },
                None => {
                    return false;
                }
            }
        }
    }

    pub fn is_curr_token(&self, kind: TokenKind) -> bool {
        match self.peek() {
            Some(Token { kind: kind_, .. }) if kind_ == &kind => true,
            _ => false,
        }
    }

    pub fn is_curr_token_doc_comment(&self) -> bool {
        matches!(self.peek(), Some(Token { kind: TokenKind::DocComment(_), .. }))
    }

    pub fn is_curr_token_pattern(&self) -> bool {
        match self.peek() {
            Some(Token { kind, .. }) => match kind {
                TokenKind::Punct(p) => match p {
                    Punct::Dollar
                    | Punct::DotDot
                    | Punct::Sub => true,
                    _ => false,
                },
                TokenKind::Identifier(_) => true,
                TokenKind::Number(_) => true,
                TokenKind::String { .. } => true,
                TokenKind::Group {
                    delim,
                    prefix: b'\0',
                    ..
                } => match delim {
                    Delim::Paren | Delim::Bracket => true,
                    Delim::Brace => false,
                },
                TokenKind::Keyword(_)
                | TokenKind::FormattedString(_)
                | TokenKind::DocComment(_)
                | TokenKind::Group { .. } => false,
            },
            _ => false,
        }
    }

    pub fn expect_ident(&mut self) -> Result<IdentWithSpan, AstError> {
        match self.peek() {
            Some(Token {
                kind: TokenKind::Identifier(ident),
                span,
            }) => {
                let ident = *ident;
                let span = *span;
                self.cursor += 1;

                Ok(IdentWithSpan(ident, span))
            },
            Some(token) => Err(AstError::unexpected_token(
                token.clone(),
                ExpectedToken::ident(),
            )),
            None => Err(AstError::unexpected_end(
                self.span_end().unwrap_or(SpanRange::dummy()),
                ExpectedToken::ident(),
            )),
        }
    }

    pub fn expect_number(&mut self) -> Result<(InternedNumeric, SpanRange), AstError> {
        match self.peek() {
            Some(Token {
                kind: TokenKind::Number(n),
                span,
            }) => {
                let n = *n;
                let span = *span;
                self.cursor += 1;

                Ok((n, span))
            },
            Some(token) => Err(AstError::unexpected_token(
                token.clone(),
                ExpectedToken::number(),
            )),
            None => Err(AstError::unexpected_end(
                self.span_end().unwrap_or(SpanRange::dummy()),
                ExpectedToken::number(),
            )),
        }
    }

    // I want it to return `&Vec<Token>`, but I can't figure out how...
    pub fn expect_group(&mut self, delim: Delim) -> Result<Vec<Token>, AstError> {
        match self.peek() {
            Some(Token { kind: TokenKind::Group { delim: delim_, tokens, prefix: b'\0' }, .. }) if *delim_ == delim => {
                let tokens = tokens.to_vec();
                self.cursor += 1;

                Ok(tokens)
            },
            Some(token) => Err(AstError::unexpected_token(
                token.clone(),
                ExpectedToken::specific(TokenKind::Group { delim, tokens: vec![], prefix: b'\0' }),
            )),
            None => Err(AstError::unexpected_end(
                self.span_end().unwrap_or(SpanRange::dummy()),
                ExpectedToken::specific(TokenKind::Group { delim, tokens: vec![], prefix: b'\0' }),
            )),
        }
    }

    pub fn expect_doc_comment(&mut self) -> Result<String, AstError> {
        match self.peek() {
            Some(Token { kind: TokenKind::DocComment(s), .. }) => Ok(s.to_string()),
            Some(token) => Err(AstError::unexpected_token(
                token.clone(),
                ExpectedToken::doc_comment(),
            )),
            None => Err(AstError::unexpected_end(
                self.span_end().unwrap_or(SpanRange::dummy()),
                ExpectedToken::doc_comment(),
            )),
        }
    }

    pub fn consume(&mut self, token_kind: TokenKind) -> Result<(), AstError> {
        match self.peek() {
            // `PartialEq` for `TokenKind` makes sense only when the kind is
            // punct, keyword, ident, or doc_comment
            Some(Token { kind, .. }) if kind == &token_kind => {
                self.cursor += 1;

                Ok(())
            },
            Some(token) => Err(AstError::unexpected_token(
                token.clone(),
                ExpectedToken::specific(token_kind),
            )),
            None => Err(AstError::unexpected_end(
                self.span_end().unwrap_or(SpanRange::dummy()),
                ExpectedToken::specific(token_kind),
            )),
        }
    }

    pub fn take_snapshot(&mut self) {
        self.snapshots.push(TokensSnapshot {
            cursor: self.cursor,
            span_end: self.span_end_,
        });
    }

    // there's no point in returning the snapshot. It only tells the caller whether
    // self.snapshots is empty or not
    pub fn pop_snapshot(&mut self) -> Result<(), ()> {
        self.snapshots.pop().map(|_| ()).ok_or(())
    }

    pub fn restore_to_last_snapshot(&mut self) {
        let last_snapshot = self.snapshots.pop().unwrap();

        self.cursor = last_snapshot.cursor;
        self.span_end_ = last_snapshot.span_end;
    }
}

// for optimization, it assumes that `Tokens.data` doesn't change.
// That's okay for now.
#[derive(Debug)]
struct TokensSnapshot {
    pub cursor: usize,
    pub span_end: Option<SpanRange>,
}
