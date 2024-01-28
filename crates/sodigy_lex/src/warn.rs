use smallvec::SmallVec;
use sodigy_error::{ExtraErrInfo, SodigyError, SodigyErrorKind};
use sodigy_intern::InternSession;
use sodigy_span::SpanRange;

#[derive(Clone)]
pub struct LexWarning {
    kind: LexWarningKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrInfo,
}

impl SodigyError<LexWarningKind> for LexWarning {
    fn get_mut_error_info(&mut self) -> &mut ExtraErrInfo {
        &mut self.extra
    }

    fn get_error_info(&self) -> &ExtraErrInfo {
        &self.extra
    }

    fn get_first_span(&self) -> Option<SpanRange> {
        self.spans.get(0).copied()
    }

    fn get_spans(&self) -> &[SpanRange] {
        &self.spans
    }

    fn err_kind(&self) -> &LexWarningKind {
        &self.kind
    }

    fn index(&self) -> u32 {
        7
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
