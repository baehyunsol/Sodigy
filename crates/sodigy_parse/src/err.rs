use sodigy_err::{ExtraErrInfo, SodigyError, SodigyErrorKind};
use sodigy_intern::InternSession;
use sodigy_number::NumericParseError;
use sodigy_span::SpanRange;

#[derive(Clone)]
pub struct ParseError {
    pub(crate) kind: ParseErrorKind,
    pub(crate) spans: Vec<SpanRange>,
    pub(crate) extra: ExtraErrInfo,
}

impl ParseError {
    pub fn unfinished_delim(c: u8, span: SpanRange) -> Self {
        ParseError {
            kind: ParseErrorKind::UnfinishedDelim(c),
            spans: vec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn mismatch_delim(c: u8, span: SpanRange) -> Self {
        ParseError {
            kind: ParseErrorKind::MismatchDelim(c),
            spans: vec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn empty_f_string(span: SpanRange) -> Self {
        ParseError {
            kind: ParseErrorKind::EmptyFString,
            spans: vec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn three_dots(span: SpanRange) -> Self {
        ParseError {
            kind: ParseErrorKind::ThreeDots,
            spans: vec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn lonely_backtick(span: SpanRange) -> Self {
        ParseError {
            kind: ParseErrorKind::LonelyBacktick,
            spans: vec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn lonely_backslash(span: SpanRange) -> Self {
        ParseError {
            kind: ParseErrorKind::LonelyBackslash,
            spans: vec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn numeric_parse_error(e: NumericParseError, span: SpanRange) -> Self {
        ParseError {
            kind: e.into(),
            spans: vec![span],
            extra: ExtraErrInfo::none(),
        }
    }
}

impl SodigyError<ParseErrorKind> for ParseError {
    fn get_error_info(&self) -> &ExtraErrInfo {
        &self.extra
    }

    fn get_mut_error_info(&mut self) -> &mut ExtraErrInfo {
        &mut self.extra
    }

    fn get_first_span(&self) -> SpanRange {
        self.spans[0]
    }

    fn get_spans(&self) -> &[SpanRange] {
        &self.spans
    }

    fn err_kind(&self) -> &ParseErrorKind {
        &self.kind
    }
}

#[derive(Clone)]
pub enum ParseErrorKind {
    UnfinishedDelim(u8),  // no end
    MismatchDelim(u8),    // no start
    EmptyFString,
    ThreeDots,
    LonelyBacktick,
    LonelyBackslash,

    // when an exp of a numeric literal is too big
    // e.g. `1.2e10000000000000000000000000`
    // exp should be a valid i64
    NumericExpOverflow,
}

impl From<NumericParseError> for ParseErrorKind {
    fn from(e: NumericParseError) -> ParseErrorKind {
        match e {
            NumericParseError::ExpOverflow => ParseErrorKind::NumericExpOverflow,
        }
    }
}

impl SodigyErrorKind for ParseErrorKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            ParseErrorKind::UnfinishedDelim(d) => format!("unclosed delimiter `{}`", *d as char),
            ParseErrorKind::MismatchDelim(d) => format!("unexpected character `{}`", *d as char),
            ParseErrorKind::EmptyFString => "empty format-string".to_string(),
            ParseErrorKind::ThreeDots => "invalid literal: `...`".to_string(),
            ParseErrorKind::NumericExpOverflow => "too large numeric literal".to_string(),
            ParseErrorKind::LonelyBacktick => "field modifier without a field name".to_string(),
            ParseErrorKind::LonelyBackslash => "unexpected character: `\\`".to_string(),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            ParseErrorKind::NumericExpOverflow => "Though Sodigy allows infinite-size integers, \
it uses 64-bit integer for its exponent. That means `123e100000` is okay, but `123e9223372036854775808` is not.".to_string(),
            ParseErrorKind::EmptyFString => "Remove the curly braces or fill it with a value.".to_string(),
            ParseErrorKind::ThreeDots => "If you are to make a range of decimal-pointed numbers, use parenthesis. \
For example, use `(3.)..4.` instead of `3...4.`.".to_string(),
            ParseErrorKind::LonelyBacktick => "You have to specify the name of the field you want to modify. A backtick character alone doesn't do anything.".to_string(),
            ParseErrorKind::UnfinishedDelim(_)
            | ParseErrorKind::MismatchDelim(_)
            | ParseErrorKind::LonelyBackslash => String::new(),
        }
    }
}
