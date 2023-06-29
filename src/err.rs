use crate::session::LocalParseSession;
use crate::span::Span;
use crate::token::TokenKind;

mod kind;
#[cfg(test)] mod tests;

pub use kind::ParseErrorKind;

/*
 * It's okay for errors to be expensive to initialize, because each parse_session is supposed to 
 * encounter at most one error...
 *
 * Avoid patterns that catches a ParseError and return Ok
 */

// Actually it's both for parser and lexer
#[derive(Debug)]
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

    pub fn eoe(span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedEoe,
            span,
            message: String::new()
        }
    }

    pub fn is_eof(&self) -> bool {
        self.kind == ParseErrorKind::UnexpectedEof
    }

    pub fn is_eoe(&self) -> bool {
        self.kind == ParseErrorKind::UnexpectedEoe
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

    pub fn tok(t: TokenKind, span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedToken(t),
            span,
            message: String::new()
        }
    }

    pub fn tok_msg(t: TokenKind, span: Span, message: String) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedToken(t),
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