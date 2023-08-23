use crate::ast::NameOrigin;
use crate::pattern::{PatternErrorKind, RangeType};
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::token::{Keyword, OpToken, Token, TokenKind};
use crate::utils::print_list;
use hmath::Ratio;
use sdg_fs::FileError;

mod kind;

// I want it to be in `sdg_type_check`, but it cannot be done because
// `sdg_ast` cannot depend on `sdg_type_check`
mod ty_err;

pub use ty_err::TypeError;

#[cfg(test)]
pub mod tests;

pub trait SodigyError {
    /// how it's shown to the programmer
    fn render_err(&self, session: &LocalParseSession) -> String;

    /// if self.message.is_empty() && the compiler thinks there's a helpful message for this error_kind,
    /// add a message
    fn try_add_more_helpful_message(&mut self);

    /// if the error doesn't have any span, it returns Span::dummy\
    /// if it has multiple ones, it returns the smallest one (Span implmenets PartialOrd)\
    /// it need not be perfect, some errors in corner cases are tolerable (though not desired)\
    /// it's used to sort the error messages
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

    pub(crate) fn lambda_hash_collision(span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::LambdaHashCollision,
            span: vec![span],
            message: String::new(),
        }
    }

    pub(crate) fn invalid_utf8(cs: Vec<u8>, span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::InvalidUTF8(cs),
            span: vec![span],
            message: String::new(),
        }
    }

    pub(crate) fn invalid_char_literal(buf: Vec<u32>, span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::InvalidCharLiteral(buf.len()),
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

    pub(crate) fn multi_def(name: InternedString, span1: Span, span2: Span, param_type: ParamType) -> Self {
        ParseError {
            kind: ParseErrorKind::MultipleDefParam(name, param_type),
            span: vec![span1, span2],
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

    pub(crate) fn non_integer_in_range(t: Token) -> Self {
        ParseError {
            kind: ParseErrorKind::InvalidPattern(PatternErrorKind::NonIntegerInRange(t.unwrap_number())),
            span: vec![t.span],
            message: String::from("Only integers are allowed in range patterns."),
        }
    }

    pub(crate) fn invalid_int_range(n1: Ratio, n2: Ratio, range_type: RangeType, span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::InvalidPattern(PatternErrorKind::InvalidIntegerRange(n1, n2, range_type)),
            span: vec![span],
            message: String::new(),
        }
    }

    pub(crate) fn invalid_char_range(c1: u32, c2: u32, range_type: RangeType, span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::InvalidPattern(PatternErrorKind::InvalidCharRange(c1, c2, range_type)),
            span: vec![span],
            message: String::new(),
        }
    }

    pub(crate) fn multiple_shorthand_in_pattern(spans: Vec<Span>) -> Self {
        ParseError {
            kind: ParseErrorKind::InvalidPattern(PatternErrorKind::MultipleShorthands),
            span: spans,
            message: String::new(),
        }
    }

    pub(crate) fn pattern_from_arg(name: InternedString, origin: NameOrigin, span: Span) -> Self {
        ParseError {
            kind: ParseErrorKind::PatternFromArg(name, origin),
            span: vec![span],
            message: String::new(),
        }
    }

    pub(crate) fn multi_field_binding_in_pattern(name: InternedString, spans: Vec<Span>) -> Self {
        ParseError {
            kind: ParseErrorKind::InvalidPattern(PatternErrorKind::MultiFieldBindingInPattern(name)),
            span: spans,
            message: String::new(),
        }
    }

    // TODO: I want to raise an actual type error
    pub(crate) fn unmatched_type_in_range(t1: Token, t2: Token) -> Self {
        todo!()
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

    pub(crate) fn set_span(&mut self, span: Vec<Span>) {
        self.span = span;
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
            } if got == &TokenKind::dotdot() => {
                self.set_msg(
                    "If you meant to use `..` as a prefix operator, try `0..a` instead of `..a`."
                );
            },
            ParseErrorKind::UnexpectedToken {
                got: TokenKind::Keyword(keyword), expected,
            } => {
                let mut is_expecting_identifier = expected == &ExpectedToken::AnyExpression
                    || expected == &ExpectedToken::AnyPattern;

                if let ExpectedToken::SpecificTokens(tokens) = expected {
                    if tokens.iter().any(
                        |t| if let TokenKind::Identifier(_) = t { true } else { false }
                    ) {
                        is_expecting_identifier = true;
                    }
                }

                if is_expecting_identifier {
                    let k = keyword.render_err();
                    self.set_msg(&format!(
                        "`{k}` is a keyword, not an identifier.\nIf you meant to use `{k}` as an identifier, try `{k}_`.",
                    ));
                }
            },
            ParseErrorKind::MultipleDefParam(_, ParamType::FuncGenericAndParam) => {
                self.set_msg(
                    "In Sodigy, types are first class objects, which means types and parameters are in the same name scope."
                );
            },
            ParseErrorKind::LambdaHashCollision => {
                self.set_msg(
                    "The compiler generates hash value for each lambda function, and there's a collision in hash values.\nThis is very rare situation, you're unlucky.\nPlease try again after inserting a whitespace or comments BEFORE the lambda function."
                );
            },
            ParseErrorKind::PatternFromArg(_, origin) => {
                self.set_msg(
                    &format!(
                        "It expected a name of an enum variant or a struct, but got {}.", 
                        origin.render_err(),
                    )
                );
            },
            ParseErrorKind::InvalidCharLiteral(len) if *len > 1 => {
                self.set_msg(
                    "If you meant to write a string literal, use double quotes."
                );
            },
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
    AnyPattern,
    AnyStatement,
    SpecificTokens(Vec<TokenKind>),
    Nothing,
}

impl ExpectedToken {
    pub fn render_err(&self, session: &LocalParseSession) -> String {
        match self {
            ExpectedToken::AnyExpression => "expected any kind of expression".to_string(),
            ExpectedToken::AnyPattern => "expected any kind of pattern".to_string(),
            ExpectedToken::Nothing => "expected no tokens".to_string(),
            ExpectedToken::AnyStatement => {
                let e = ExpectedToken::SpecificTokens(vec![
                    TokenKind::Operator(OpToken::At),
                    TokenKind::Keyword(Keyword::Use),
                    TokenKind::Keyword(Keyword::Def),
                    TokenKind::Keyword(Keyword::Module),
                    TokenKind::Keyword(Keyword::Enum),
                ]);

                e.render_err(session)
            },
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
            | (ExpectedToken::Nothing, ExpectedToken::Nothing)
            | (ExpectedToken::AnyPattern, ExpectedToken::AnyPattern) => true,

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
