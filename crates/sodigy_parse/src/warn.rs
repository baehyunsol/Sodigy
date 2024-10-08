use smallvec::{SmallVec, smallvec};
use sodigy_error::{
    ErrorContext,
    ExtraErrorInfo,
    SodigyError,
    SodigyErrorKind,
    Stage,
};
use sodigy_intern::InternSession;
use sodigy_span::SpanRange;

mod endec;

#[derive(Clone)]
pub struct ParseWarning {
    kind: ParseWarningKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrorInfo,
}

impl ParseWarning {
    pub fn nothing_to_eval_in_fstring(span: SpanRange) -> Self {
        ParseWarning {
            kind: ParseWarningKind::NothingToEvalInFString,
            spans: smallvec![span],
            extra: ExtraErrorInfo::at_context(ErrorContext::ParsingFormattedString),
        }
    }

    pub fn unmatched_curly_brace(span: SpanRange) -> Self {
        ParseWarning {
            kind: ParseWarningKind::UnmatchedCurlyBrace,
            spans: smallvec![span],
            extra: ExtraErrorInfo::at_context(ErrorContext::ParsingFormattedString),
        }
    }
}

impl SodigyError<ParseWarningKind> for ParseWarning {
    fn get_mut_error_info(&mut self) -> &mut ExtraErrorInfo {
        &mut self.extra
    }

    fn get_error_info(&self) -> &ExtraErrorInfo {
        &self.extra
    }

    fn get_first_span(&self) -> Option<SpanRange> {
        self.spans.get(0).copied()
    }

    fn get_spans(&self) -> &[SpanRange] {
        &self.spans
    }

    fn error_kind(&self) -> &ParseWarningKind {
        &self.kind
    }

    fn is_warning(&self) -> bool {
        true
    }

    fn index(&self) -> u32 {
        11
    }

    fn get_stage(&self) -> Stage {
        Stage::Parse
    }
}

#[derive(Clone)]
pub enum ParseWarningKind {
    NothingToEvalInFString,
    UnmatchedCurlyBrace,
}

impl SodigyErrorKind for ParseWarningKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            ParseWarningKind::NothingToEvalInFString => String::from("nothing to evaluate in a formatted string"),
            ParseWarningKind::UnmatchedCurlyBrace => String::from("unmatched curly brace in a formatted string"),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            ParseWarningKind::NothingToEvalInFString => String::from("Try remove `f`."),
            ParseWarningKind::UnmatchedCurlyBrace => String::new(),
        }
    }

    fn index(&self) -> u32 {
        match self {
            ParseWarningKind::NothingToEvalInFString => 0,
            ParseWarningKind::UnmatchedCurlyBrace => 1,
        }
    }
}
