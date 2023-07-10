use crate::session::LocalParseSession;
use crate::span::Span;
use crate::token::TokenKind;

mod kind;

#[cfg(test)]
pub mod tests;

pub trait SodigyError {
    fn render_err(&self, session: &LocalParseSession) -> String;
}

pub use kind::ParseErrorKind;

/*
 * The compiler assumes that a successful compilation never initializes a `T: SodigyError`.
 * That's why it's okay for `T: SodigyError` and its related functions to be expensive.
 * Please try not to break this assumption.
 */

/// Actually it's both for parser and lexer
pub struct ParseError {
    pub(crate) kind: ParseErrorKind,
    pub(crate) span: Span,

    // At least one of `ParseError`'s field must be private
    // I'll someday implement a checker that `ParseError` is initialized at most once during a compilation
    // To do that, I have to make sure that all the other players use constructor functions, instead of direct initialization
    message: String,
}

impl ParseError {
    // `span` must point to the start of the token it's parsing, not just the end of the file
    pub(crate) fn eof(span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedEof,
            span,
            message: String::new(),
        }
    }

    pub(crate) fn eoe(span: Span, expected: ExpectedToken) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedEoe(expected),
            span,
            message: String::new(),
        }
    }

    pub(crate) fn eoe_msg(span: Span, expected: ExpectedToken, message: String) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedEoe(expected),
            span,
            message,
        }
    }

    pub(crate) fn is_eof(&self) -> bool {
        self.kind == ParseErrorKind::UnexpectedEof
    }

    pub(crate) fn is_eoe(&self) -> bool {
        if let ParseErrorKind::UnexpectedEoe(_) = self.kind {
            true
        } else {
            false
        }
    }

    pub(crate) fn is_iutf8(&self) -> bool {
        if let ParseErrorKind::InvalidUTF8(_) = self.kind {
            true
        } else {
            false
        }
    }

    pub(crate) fn ch(c: char, span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedChar(c),
            span,
            message: String::new(),
        }
    }

    pub(crate) fn ch_msg(c: char, span: Span, message: String) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedChar(c),
            span,
            message,
        }
    }

    pub(crate) fn tok(got: TokenKind, span: Span, expected: ExpectedToken) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedToken { got, expected },
            span,
            message: String::new(),
        }
    }

    pub(crate) fn tok_msg(got: TokenKind, span: Span, expected: ExpectedToken, message: String) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedToken { got, expected },
            span,
            message,
        }
    }

    pub(crate) fn invalid_utf8(cs: Vec<u8>, ind: usize) -> Self {
        ParseError {
            kind: ParseErrorKind::InvalidUTF8(cs),
            span: Span::dummy_index(ind),
            message: String::new(),
        }
    }

    pub(crate) fn set_span_of_eof(mut self, span: Span) -> Self {
        if (self.is_eof() || self.is_eoe()) && self.span.is_dummy() {
            self.span = span;
            self
        } else {
            self
        }
    }

    pub(crate) fn set_ind_and_fileno(mut self, span: Span) -> Self {
        if self.is_iutf8() && self.span.is_dummy_index() {
            let offset = self.span.index;
            self.span = span;
            self.span.index += offset;

            self
        } else {
            self
        }
    }
}

impl SodigyError for ParseError {
    fn render_err(&self, session: &LocalParseSession) -> String {
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
    Nothing,
}

impl ExpectedToken {
    pub fn render_err(&self, session: &LocalParseSession) -> String {
        match self {
            ExpectedToken::AnyExpression => "Any kind of expression was expected,".to_string(),
            ExpectedToken::Nothing => "Expected no tokens,".to_string(),
            ExpectedToken::SpecificTokens(token_kinds) => {
                format!("Expected {},", pretty_list(token_kinds, session))
            }
        }
    }

    #[cfg(test)]
    pub fn is_same_type(&self, other: &ExpectedToken) -> bool {
        match (self, other) {
            (ExpectedToken::AnyExpression, ExpectedToken::AnyExpression)
            | (ExpectedToken::Nothing, ExpectedToken::Nothing) => true,

            // for test runners, the order of the tokens do not matter
            // for test functions, we do not have to care about their performance... really?
            (ExpectedToken::SpecificTokens(tokens1), ExpectedToken::SpecificTokens(tokens2)) => {

                for t1 in tokens1.iter() {

                    if !tokens2.iter().any(|t2| t1.is_same_type(t2)) {
                        return false;
                    }

                }

                true
            }
            _ => false,
        }
    }
}

// As stated at the top, error-related functions are okay to be slow
fn pretty_list(token_kinds: &[TokenKind], session: &LocalParseSession) -> String {
    if token_kinds.len() == 0 {
        String::new()
    } else if token_kinds.len() == 1 {
        token_kinds[0].render_err(session)
    } else if token_kinds.len() == 2 {
        format!(
            "{} or {}",
            token_kinds[0].render_err(session),
            token_kinds[1].render_err(session)
        )
    } else {
        format!(
            "{}, {}",
            token_kinds[0].render_err(session),
            pretty_list(&token_kinds[1..], session)
        )
    }
}
