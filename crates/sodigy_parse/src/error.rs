use crate::token_tree::{TokenTree, TokenTreeKind};
use smallvec::{smallvec, SmallVec};
use sodigy_error::{
    ExpectedToken,
    ExtraErrorInfo,
    RenderError,
    SodigyError,
    SodigyErrorKind,
    Stage,
};
use sodigy_intern::InternSession;
use sodigy_span::SpanRange;

mod endec;

#[derive(Clone)]
pub struct ParseError {
    pub(crate) kind: ParseErrorKind,
    pub(crate) spans: SmallVec<[SpanRange; 1]>,
    pub(crate) extra: ExtraErrorInfo,
}

impl ParseError {
    pub fn unfinished_delim(c: u8, span: SpanRange) -> Self {
        ParseError {
            kind: ParseErrorKind::UnfinishedDelim(c),
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn mismatch_delim(c: u8, span: SpanRange) -> Self {
        ParseError {
            kind: ParseErrorKind::MismatchDelim(c),
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn empty_fstring(span: SpanRange) -> Self {
        ParseError {
            kind: ParseErrorKind::EmptyFString,
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn three_dots(span: SpanRange) -> Self {
        ParseError {
            kind: ParseErrorKind::ThreeDots,
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn lonely_backtick(span: SpanRange) -> Self {
        ParseError {
            kind: ParseErrorKind::LonelyBacktick,
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn lonely_backslash(span: SpanRange) -> Self {
        ParseError {
            kind: ParseErrorKind::LonelyBackslash,
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn fstring_single_quote(span: SpanRange) -> Self {
        ParseError {
            kind: ParseErrorKind::FStringSingleQuote,
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn fstring_without_prefix(has_prefix_b: bool, span: SpanRange) -> Self {
        ParseError {
            kind: ParseErrorKind::FStringWithoutPrefix { has_prefix_b },
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn unexpected_token(token: TokenTree, expected_token: ExpectedToken<TokenTreeKind>) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedToken(token.kind, expected_token),
            spans: smallvec![token.span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn unexpected_eof(expected_token: ExpectedToken<TokenTreeKind>, span: SpanRange) -> Self {
        ParseError {
            kind: ParseErrorKind::UnexpectedEof(expected_token),
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }
}

impl SodigyError<ParseErrorKind> for ParseError {
    fn get_error_info(&self) -> &ExtraErrorInfo {
        &self.extra
    }

    fn get_mut_error_info(&mut self) -> &mut ExtraErrorInfo {
        &mut self.extra
    }

    fn get_first_span(&self) -> Option<SpanRange> {
        self.spans.get(0).copied()
    }

    fn get_spans(&self) -> &[SpanRange] {
        &self.spans
    }

    fn error_kind(&self) -> &ParseErrorKind {
        &self.kind
    }

    fn index(&self) -> u32 {
        10
    }

    fn get_stage(&self) -> Stage {
        Stage::Parse
    }
}

#[derive(Clone)]
pub enum ParseErrorKind {
    UnfinishedDelim(u8),  // no end
    MismatchDelim(u8),    // no start
    EmptyFString,
    FStringSingleQuote,
    FStringWithoutPrefix { has_prefix_b: bool },
    ThreeDots,
    LonelyBacktick,
    LonelyBackslash,
    UnexpectedToken(TokenTreeKind, ExpectedToken<TokenTreeKind>),
    UnexpectedEof(ExpectedToken<TokenTreeKind>),
}

impl SodigyErrorKind for ParseErrorKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            ParseErrorKind::UnfinishedDelim(d) => format!("unclosed delimiter `{}`", *d as char),
            ParseErrorKind::MismatchDelim(d) => format!("unexpected character `{}`", *d as char),
            ParseErrorKind::EmptyFString => "empty format-string".to_string(),
            ParseErrorKind::ThreeDots => "invalid literal: `...`".to_string(),
            ParseErrorKind::LonelyBacktick => "field modifier without a field name".to_string(),
            ParseErrorKind::LonelyBackslash => "unexpected character `\\`".to_string(),
            ParseErrorKind::FStringSingleQuote => "format-string with single quotes".to_string(),
            ParseErrorKind::FStringWithoutPrefix {
                has_prefix_b
            } => if *has_prefix_b {
                "format-string with a prefix `b`"
            } else {
                "format-string without a prefix `f`"
            }.to_string(),
            ParseErrorKind::UnexpectedToken(token, expected) => format!("expected {expected}, got `{}`", token.render_error()),
            ParseErrorKind::UnexpectedEof(expected) => format!("expected {expected}, got nothing"),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            ParseErrorKind::EmptyFString => "Remove the curly braces or fill it with a value.".to_string(),
            ParseErrorKind::ThreeDots => "If you are to make a range of decimal-pointed numbers, use parenthesis. \
For example, use `(3.)..4.` instead of `3...4.`.".to_string(),
            ParseErrorKind::LonelyBacktick => "You have to specify the name of the field you want to modify. A backtick character alone doesn't do anything.".to_string(),
            ParseErrorKind::FStringSingleQuote => "Use `f\"...\"` instead of `f'...'`.".to_string(),
            ParseErrorKind::FStringWithoutPrefix {
                has_prefix_b
            } => if *has_prefix_b {
                "A format-literal `\\{` is not allowed in a binary literal. Try `\\\\{` to escaped the backslash character."
            } else {
                "Add `f` before `\"`."
            }.to_string(),
            ParseErrorKind::UnfinishedDelim(_)
            | ParseErrorKind::MismatchDelim(_)
            | ParseErrorKind::LonelyBackslash
            | ParseErrorKind::UnexpectedToken(_, _)
            | ParseErrorKind::UnexpectedEof(_) => String::new(),
        }
    }

    fn index(&self) -> u32 {
        match self {
            ParseErrorKind::UnfinishedDelim(_) => 0,
            ParseErrorKind::MismatchDelim(_) => 1,
            ParseErrorKind::EmptyFString => 2,
            ParseErrorKind::FStringSingleQuote => 3,
            ParseErrorKind::FStringWithoutPrefix { .. } => 4,
            ParseErrorKind::ThreeDots => 5,
            ParseErrorKind::LonelyBacktick => 6,
            ParseErrorKind::LonelyBackslash => 7,
            ParseErrorKind::UnexpectedToken(_, _) => 8,
            ParseErrorKind::UnexpectedEof(_) => 9,
        }
    }
}
