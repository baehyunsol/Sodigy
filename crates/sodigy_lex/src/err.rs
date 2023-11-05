use smallvec::{smallvec, SmallVec};
use sodigy_err::{concat_commas, ExtraErrInfo, SodigyError, SodigyErrorKind};
use sodigy_span::SpanRange;

use crate::QuoteKind;
use crate::num::err::ParseNumberError;

#[derive(Clone)]
pub struct LexError {
    kind: LexErrorKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrInfo,
}

impl LexError {
    pub fn unexpected_char(c: char, span: SpanRange) -> Self {
        LexError {
            kind: LexErrorKind::UnexpectedChar(c, ExpectedChars::Any),
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn invalid_utf8(span: SpanRange) -> Self {
        LexError {
            kind: LexErrorKind::InvalidUtf8,
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn unfinished_comment(span: SpanRange) -> Self {
        LexError {
            kind: LexErrorKind::UnfinishedComment,
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn unfinished_string(kind: QuoteKind, span: SpanRange) -> Self {
        LexError {
            kind: LexErrorKind::UnfinishedString(kind),
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn unfinished_num_literal(span: SpanRange) -> Self {
        LexError {
            kind: LexErrorKind::UnfinishedNumLiteral(ExpectedChars::Any),
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn parse_num_error(e: ParseNumberError, span: SpanRange) -> Self {
        LexError {
            kind: e.into(),
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn set_expected_chars(&mut self, chars: Vec<u8>) -> &mut Self {
        match &mut self.kind {
            LexErrorKind::UnexpectedChar(_, e)
            | LexErrorKind::UnfinishedNumLiteral(e) => {
                *e = ExpectedChars::Specific(chars);
            },
            _ => {
                #[cfg(test)] unreachable!();
            },
        }

        self
    }
}

impl SodigyError<LexErrorKind> for LexError {
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

    fn err_kind(&self) -> &LexErrorKind {
        &self.kind
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExpectedChars {
    Any,
    Specific(Vec<u8>),
}

impl ExpectedChars {
    pub fn list(&self) -> String {
        match self {
            ExpectedChars::Any => "any valid token".to_string(),
            ExpectedChars::Specific(ts) => concat_commas(
                &ts.iter().map(|c| format!("{c}")).collect::<Vec<String>>(),
                "or",
                "",  // prefix
                "",  // suffix
            ),
        }
    }
}

#[derive(Clone)]
pub enum LexErrorKind {
    InvalidUtf8,
    UnexpectedChar(char, ExpectedChars),
    UnfinishedComment,  // must be CommentKind::Multi
    UnfinishedString(QuoteKind),
    UnfinishedNumLiteral(ExpectedChars),
}

impl From<ParseNumberError> for LexErrorKind {
    fn from(e: ParseNumberError) -> LexErrorKind {
        match e {
            ParseNumberError::UnfinishedNumLiteral(ex) => LexErrorKind::UnfinishedNumLiteral(ex),
        }
    }
}

use sodigy_intern::InternSession;

impl SodigyErrorKind for LexErrorKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            LexErrorKind::InvalidUtf8 => "invalid utf-8".to_string(),
            LexErrorKind::UnexpectedChar(c, e) => format!("expected {}, got `{c}`", e.list()),
            LexErrorKind::UnfinishedComment => "unterminated block comment".to_string(),
            LexErrorKind::UnfinishedString(q) => if *q == QuoteKind::Double {
                "unterminated string literal"
            } else {
                "unterminated character literal"
            }.to_string(),
            LexErrorKind::UnfinishedNumLiteral(_) => "unterminated numeric literal".to_string(),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        String::new()
    }
}
