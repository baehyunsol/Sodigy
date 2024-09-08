use crate::IdentWithSpan;
use crate::error::AstError;
use crate::{Token, TokenKind};
use sodigy_error::ExpectedToken;
use sodigy_intern::{InternedNumeric, InternedString};
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

    /// if self.data.is_empty, self.span_end() returns this span
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

    pub fn get_cursor(&self) -> usize {
        self.cursor
    }

    pub fn step(&mut self) -> Option<&Token> {
        self.cursor += 1;
        self.data.get(self.cursor - 1)
    }

    /// It returns Err if it cannot go backward
    pub fn backward(&mut self) -> Result<(), ()> {
        if self.cursor == 0 {
            Err(())
        }

        else {
            self.cursor -= 1;
            Ok(())
        }
    }

    /// It's the last span of the last token.
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
                        Keyword::Let
                        | Keyword::Module
                        | Keyword::Import => {
                            return true;
                        },
                        Keyword::If
                        | Keyword::Else
                        | Keyword::As
                        | Keyword::From
                        | Keyword::Pattern
                        | Keyword::Enum
                        | Keyword::Struct
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
                    },
                },
                None => {
                    return false;
                },
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
                | TokenKind::Group { .. }

                // `TokenKind::Macro`s are all lowered before this stage, so we don't need to worry about this variant
                | TokenKind::Macro { .. } => false,
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

    pub fn expect_doc_comment(&mut self) -> Result<InternedString, AstError> {
        match self.peek() {
            Some(Token { kind: TokenKind::DocComment(s), .. }) => Ok(*s),
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

    // some functions rely on the fact that this method moves the cursor
    // if and only if there's an expected token
    // please do not change this behavior
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

    pub fn get_previous_generic_span(&self) -> Option<SpanRange> {
        let mut cursor = self.cursor;
        let mut stack = 0;
        let mut span_start = None;
        let mut span_end = None;

        while cursor > 0 {
            cursor -= 1;

            match self.data.get(cursor) {
                Some(Token {
                    kind: TokenKind::Punct(Punct::Gt),
                    span,
                }) => {
                    stack += 1;

                    if stack == 1 {
                        span_end = Some(*span);
                    }
                },
                Some(Token {
                    kind: TokenKind::Punct(Punct::Lt),
                    span,
                }) => {
                    // unmatched
                    if stack == 0 {
                        return None;
                    }

                    stack -= 1;

                    if stack == 0 {
                        span_start = Some(*span);
                        break;
                    }
                },
                Some(_) => {},
                None => {
                    break;
                },
            }
        }

        match (span_start, span_end) {
            (Some(start), Some(end)) => Some(start.merge(end)),
            _ => None,
        }
    }

    pub fn first_few_tokens(&self) -> &[Token] {
        &self.data[..4.min(self.data.len())]
    }

    pub fn match_first_tokens(
        &self,
        tokens: &[TokenKind],
    ) -> bool {
        let mut curr_index = self.cursor;

        for token in tokens.iter() {
            if let Some(curr_token) = self.data.get(curr_index) {
                if curr_token.kind != *token {
                    return false;
                }
            }

            else {
                return false;
            }

            curr_index += 1;
        }

        true
    }
}

// for optimization, it assumes that `Tokens.data` doesn't change.
// That's okay for now.
#[derive(Debug)]
struct TokensSnapshot {
    pub cursor: usize,
    pub span_end: Option<SpanRange>,
}
