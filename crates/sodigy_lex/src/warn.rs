use smallvec::SmallVec;
use sodigy_error::{ExtraErrorInfo, SodigyError, SodigyErrorKind, Stage};
use sodigy_intern::InternSession;
use sodigy_span::SpanRange;

#[derive(Clone)]
pub struct LexWarning {
    kind: LexWarningKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrorInfo,
}

impl SodigyError<LexWarningKind> for LexWarning {
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

    fn error_kind(&self) -> &LexWarningKind {
        &self.kind
    }

    fn is_warning(&self) -> bool {
        true
    }

    fn index(&self) -> u32 {
        7
    }

    fn get_stage(&self) -> Stage {
        Stage::Lex
    }
}

#[derive(Clone)]
pub enum LexWarningKind {}

// all of these are unreachable because the lexer never emits any warning
impl SodigyErrorKind for LexWarningKind {
    fn msg(&self, _: &mut InternSession) -> String {
        unreachable!()
    }

    fn help(&self, _: &mut InternSession) -> String {
        unreachable!()
    }

    fn index(&self) -> u32 {
        unreachable!()
    }
}
