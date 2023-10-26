use crate::IdentWithSpan;
use crate::err::{ExpectedToken, AstError};
use crate::{Token, TokenKind};
use sodigy_keyword::Keyword;
use sodigy_parse::{Delim, Punct};
use sodigy_span::SpanRange;

#[derive(Debug)]
pub struct Tokens<'a> {
    data: &'a mut Vec<Token>,
    cursor: usize,
}

impl<'a> Tokens<'a> {
    pub fn from_vec(data: &'a mut Vec<Token>) -> Self {
        Tokens {
            data,
            cursor: 0,
        }
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
        self.data.last().map(|t| t.span.last_char())
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
                        | Keyword::Use => {
                            return true;
                        },
                        Keyword::If
                        | Keyword::Else
                        | Keyword::As
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
}
