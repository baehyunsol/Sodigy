use crate::session::LocalParseSession;
use crate::span::Span;
use crate::token::TokenKind;

mod kind;
#[cfg(test)] mod tests;

pub use kind::ParseErrorKind;

/*
 * The compiler assumes that a successful compilation never initializes a `ParseError`.
 * That's why it's okay for `ParseError` and its related functions to be expensive.
 * Please try not to break its assumption.
 */

// Actually it's both for parser and lexer
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: Span,
    pub message: String
}

impl ParseError {

    // `span` must point to the start of the token it's parsing, not just the end of the file
    pub fn eof(span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedEof,
            span,
            message: String::new()
        }
    }

    pub fn eoe(span: Span, expected: ExpectedToken) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedEoe(expected),
            span,
            message: String::new()
        }
    }

    pub fn eoe_msg(span: Span, expected: ExpectedToken, message: String) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedEoe(expected),
            span,
            message
        }
    }

    pub fn is_eof(&self) -> bool {
        self.kind == ParseErrorKind::UnexpectedEof
    }

    pub fn is_eoe(&self) -> bool {
        if let ParseErrorKind::UnexpectedEoe(_) = self.kind { true } else { false }
    }

    pub fn ch(c: char, span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedChar(c),
            span,
            message: String::new()
        }
    }

    pub fn ch_msg(c: char, span: Span, message: String) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedChar(c),
            span, message
        }
    }

    pub fn tok(got: TokenKind, span: Span, expected: ExpectedToken) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedToken{ got, expected },
            span,
            message: String::new()
        }
    }

    pub fn tok_msg(got: TokenKind, span: Span, expected: ExpectedToken, message: String) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedToken{ got, expected },
            span, message
        }
    }

    pub fn set_span_of_eof(mut self, span: Span) -> ParseError {
    
        if (self.is_eof() || self.is_eoe()) && self.span.is_dummy() {
            self.span = span;
            self
        }

        else {
            self
        }

    }

    pub fn render_err(&self, session: &LocalParseSession) -> String {
        format!(
            "{}{}\n{}",
            self.kind.render_err(session),
            if self.message.len() > 0 {
                format!("\n{}", self.message)
            } else {
                String::new()
            },
            self.span.render_err(session)
        )
    }

}

#[derive(PartialEq)]
pub enum ExpectedToken {
    AnyExpression,
    SpecificTokens(Vec<TokenKind>),
    Nothing
}

impl ExpectedToken {

    pub fn render_err(&self, session: &LocalParseSession) -> String {

        match self {
            ExpectedToken::AnyExpression => "Any kind of expression was expected,".to_string(),
            ExpectedToken::Nothing => "Expected no tokens,".to_string(),
            ExpectedToken::SpecificTokens(token_kinds) => format!("Expected {},", pretty_list(token_kinds, session))
        }

    }

}

// As stated at the top, error-related functions are okay to be slow
fn pretty_list(token_kinds: &[TokenKind], session: &LocalParseSession) -> String {

    if token_kinds.len() == 0 {
        String::new()
    }

    else if token_kinds.len() == 1 {
        token_kinds[0].render_err(session)
    }

    else if token_kinds.len() == 2 {
        format!("{} or {}", token_kinds[0].render_err(session), token_kinds[1].render_err(session))
    }

    else {
        format!(
            "{}, {}",
            token_kinds[0].render_err(session),
            pretty_list(&token_kinds[1..], session)
        )
    }

}