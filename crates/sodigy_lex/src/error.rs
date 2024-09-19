use smallvec::{smallvec, SmallVec};
use sodigy_error::{
    concat_commas,
    ExtraErrorInfo,
    SodigyError,
    SodigyErrorKind,
    Stage,
};
use sodigy_intern::InternSession;
use sodigy_span::SpanRange;

use crate::QuoteKind;
use crate::num::error::ParseNumberError;

#[derive(Clone)]
pub struct LexError {
    kind: LexErrorKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrorInfo,
}

impl LexError {
    pub fn unexpected_char(c: char, span: SpanRange) -> Self {
        LexError {
            kind: LexErrorKind::UnexpectedChar(c, ExpectedChars::Any),
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn invalid_utf8(span: SpanRange) -> Self {
        LexError {
            kind: LexErrorKind::InvalidUtf8,
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn invalid_character_escape(c: u8, span: SpanRange) -> Self {
        LexError {
            kind: LexErrorKind::InvalidCharacterEscape(c),
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn unfinished_comment(span: SpanRange) -> Self {
        LexError {
            kind: LexErrorKind::UnfinishedComment,
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn unfinished_string(kind: QuoteKind, span: SpanRange) -> Self {
        LexError {
            kind: LexErrorKind::UnfinishedString(kind),
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn unfinished_num_literal(span: SpanRange) -> Self {
        LexError {
            kind: LexErrorKind::UnfinishedNumLiteral(ExpectedChars::Any),
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn unfinished_fstring(span: SpanRange) -> Self {
        LexError {
            kind: LexErrorKind::UnfinishedFString,
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn parse_num_error(e: ParseNumberError, span: SpanRange) -> Self {
        LexError {
            kind: e.into(),
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
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

    fn error_kind(&self) -> &LexErrorKind {
        &self.kind
    }

    fn index(&self) -> u32 {
        6
    }

    fn get_stage(&self) -> Stage {
        Stage::Lex
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
                &ts.iter().map(
                    |c| format!(
                        "{:?}",
                        char::from_u32(*c as u32).unwrap_or('�')
                    )
                ).collect::<Vec<String>>(),
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
    InvalidCharacterEscape(u8),
    UnexpectedChar(char, ExpectedChars),
    UnfinishedComment,  // must be CommentKind::Multi
    UnfinishedString(QuoteKind),
    UnfinishedNumLiteral(ExpectedChars),
    UnfinishedFString,
}

impl From<ParseNumberError> for LexErrorKind {
    fn from(e: ParseNumberError) -> LexErrorKind {
        match e {
            ParseNumberError::UnfinishedNumLiteral(ex) => LexErrorKind::UnfinishedNumLiteral(ex),
        }
    }
}

impl SodigyErrorKind for LexErrorKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            LexErrorKind::InvalidUtf8 => String::from("invalid utf-8"),
            LexErrorKind::InvalidCharacterEscape(c) => format!("invalid character escape: `\\{}`", *c as char),
            LexErrorKind::UnexpectedChar(c, e) => format!(
                "expected character {}, got character {c:?}",
                e.list(),
            ),
            LexErrorKind::UnfinishedComment => String::from("unterminated block comment"),
            LexErrorKind::UnfinishedString(q) => if *q == QuoteKind::Double {
                "unterminated string literal"
            } else {
                "unterminated character literal"
            }.to_string(),
            LexErrorKind::UnfinishedNumLiteral(_) => String::from("unterminated numeric literal"),
            LexErrorKind::UnfinishedFString => String::from("unterminated `\\{` in a literal"),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            LexErrorKind::UnfinishedFString => String::from("If you want a `\\` character and a `{` character, try `\\\\{`."),
            LexErrorKind::InvalidCharacterEscape(c) => format!("If you want a `\\` character and a `{}` character, try `\\\\{}`.", *c as char, *c as char),
            LexErrorKind::InvalidUtf8
            | LexErrorKind::UnexpectedChar(_, _)
            | LexErrorKind::UnfinishedComment
            | LexErrorKind::UnfinishedString(_)
            | LexErrorKind::UnfinishedNumLiteral(_) => String::new(),
        }
    }

    fn index(&self) -> u32 {
        match self {
            LexErrorKind::InvalidUtf8 => 0,
            LexErrorKind::InvalidCharacterEscape(_) => 1,
            LexErrorKind::UnexpectedChar(_, _) => 2,
            LexErrorKind::UnfinishedComment => 3,
            LexErrorKind::UnfinishedString(_) => 4,
            LexErrorKind::UnfinishedNumLiteral(_) => 5,
            LexErrorKind::UnfinishedFString => 6,
        }
    }
}
