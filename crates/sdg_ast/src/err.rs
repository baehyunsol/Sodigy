use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::token::{OpToken, TokenKind};
use crate::utils::print_list;
use sdg_fs::FileError;

mod kind;

#[cfg(test)]
pub mod tests;

pub trait SodigyError {
    /// how it's shown to the programmer
    fn render_err(&self, session: &LocalParseSession) -> String;

    /// if self.message.is_empty() && the compiler thinks there's a helpful message for this error_kind
    /// add a message
    fn try_add_more_helpful_message(&mut self);

    /// if the error doesn't have any span, it returns Span::dummy,
    /// if it has multiple ones, it returns the smallest one (Span implmenets PartialOrd),
    /// it need not be perfect, some errors in corner cases are tolerable (though not desired)
    fn get_first_span(&self) -> Span;
}

pub use kind::{ParamType, ParseErrorKind};

/// Actually it's both for parser and lexer
#[derive(Clone)]
pub struct ParseError {
    pub(crate) kind: ParseErrorKind,
    pub(crate) span: Vec<Span>,
    message: String,
}

impl ParseError {
    // `span` must point to the start of the token it's parsing, not just the end of the file
    pub(crate) fn eof(span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedEof,
            span: vec![span],
            message: String::new(),
        }
    }

    pub(crate) fn eof_msg(span: Span, message: String) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedEof,
            span: vec![span],
            message,
        }
    }

    pub(crate) fn eoe(span: Span, expected: ExpectedToken) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedEoe(expected),
            span: vec![span],
            message: String::new(),
        }
    }

    pub(crate) fn eoe_msg(span: Span, expected: ExpectedToken, message: String) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedEoe(expected),
            span: vec![span],
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
            span: vec![span],
            message: String::new(),
        }
    }

    pub(crate) fn ch_msg(c: char, span: Span, message: String) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedChar(c),
            span: vec![span],
            message,
        }
    }

    pub(crate) fn tok(got: TokenKind, span: Span, expected: ExpectedToken) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedToken { got, expected },
            span: vec![span],
            message: String::new(),
        }
    }

    pub(crate) fn tok_msg(got: TokenKind, span: Span, expected: ExpectedToken, message: String) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedToken { got, expected },
            span: vec![span],
            message,
        }
    }

    pub(crate) fn invalid_utf8(cs: Vec<u8>, span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::InvalidUTF8(cs),
            span: vec![span],
            message: String::new(),
        }
    }

    pub(crate) fn invalid_utf8_dummy_index(cs: Vec<u8>, ind: usize) -> Self {
        ParseError {
            kind: ParseErrorKind::InvalidUTF8(cs),
            span: vec![Span::dummy_index(ind)],
            message: String::new(),
        }
    }

    pub(crate) fn untyped_arg(arg_name: InternedString, func_name: InternedString, span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::UntypedArg(arg_name, func_name),
            span: vec![span],
            message: String::new(),
        }
    }

    pub(crate) fn multi_def(name: InternedString, span: Span, param_type: ParamType) -> Self {
        ParseError {
            kind: ParseErrorKind::MultipleDefParam(name, param_type),
            span: vec![span],
            message: String::new(),
        }
    }

    pub(crate) fn file(file_error: FileError) -> Self {
        ParseError {
            kind: ParseErrorKind::FileError(file_error),
            span: vec![],
            message: String::new(),
        }
    }

    pub(crate) fn set_span_of_eof(mut self, span: Span) -> Self {
        if (self.is_eof() || self.is_eoe()) && (self.span.is_empty() || self.span[0].is_dummy()) {
            self.span = vec![span];
            self
        } else {
            self
        }
    }

    pub(crate) fn set_ind_and_fileno(mut self, span: Span) -> Self {
        if self.is_iutf8() && self.span[0].is_dummy_index() {
            assert_eq!(self.span.len(), 1, "Internal Compiler Error D8DFF0DF984");

            let offset = self.span[0].start;
            self.span[0] = span;
            self.span[0].start += offset;
            self.span[0].end += offset;

            self
        } else {
            self
        }
    }

    pub(crate) fn is_unexpected_token(&self) -> bool {
        if let ParseErrorKind::UnexpectedToken{ .. } = &self.kind {
            true
        } else {
            false
        }
    }

    pub(crate) fn set_msg(&mut self, message: &str) {
        self.message = message.to_string();
    }

    pub(crate) fn set_expected_tokens_instead_of_nothing(&mut self, tokens: Vec<TokenKind>) {
        match &mut self.kind {
            ParseErrorKind::UnexpectedToken { expected, .. } if expected == &ExpectedToken::Nothing => {
                *expected = ExpectedToken::SpecificTokens(tokens);
            },
            _ => {},
        }
    }
}

impl SodigyError for ParseError {
    fn render_err(&self, session: &LocalParseSession) -> String {
        let mut e = self.clone();
        e.try_add_more_helpful_message();

        format!(
            "Error: {}{}{}",
            e.kind.render_err(session),
            if e.message.len() > 0 {
                format!("\n{}", e.message)
            } else {
                String::new()
            },
            e.span.iter().map(
                |span| format!("\n{}", span.render_err(session))
            ).collect::<Vec<String>>().concat(),
        )
    }

    fn try_add_more_helpful_message(&mut self) {
        if !self.message.is_empty() {
            return;
        }

        match &self.kind {
            ParseErrorKind::UnexpectedToken {
                got,
                expected: ExpectedToken::AnyExpression,
            } if got == &TokenKind::Operator(OpToken::DotDot) => {
                self.set_msg(
                    "If you want to use `..` as a prefix operator, try `0..a` instead of `..a`."
                );
            },
            ParseErrorKind::MultipleDefParam(_, ParamType::FuncGenericAndParam) => {
                self.set_msg(
                    "In Sodigy, types are first class objects, which means types and parameters are in the same name scope."
                );
            }
            _ => {}
        }
    }

    fn get_first_span(&self) -> Span {
        if self.span.is_empty() {
            Span::dummy()
        } else {
            let mut curr = self.span[0];

            for span in self.span.iter() {
                if *span < curr {
                    curr = *span;
                }
            }

            curr
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ExpectedToken {
    AnyExpression,
    SpecificTokens(Vec<TokenKind>),
    Nothing,
}

impl ExpectedToken {
    pub fn render_err(&self, session: &LocalParseSession) -> String {
        match self {
            ExpectedToken::AnyExpression => "expected any kind of expression".to_string(),
            ExpectedToken::Nothing => "expected no tokens".to_string(),
            ExpectedToken::SpecificTokens(token_kinds) => {
                format!(
                    "expected {}",
                    print_list(
                        &token_kinds.iter().map(
                            |kind| kind.render_err(session)
                        ).collect::<Vec<String>>(),
                        "", "", "or"
                    ),
                )
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
