use smallvec::SmallVec;
use sodigy_error::{
    ExtraErrorInfo,
    SodigyError,
    SodigyErrorKind,
    Stage,
};
use sodigy_intern::InternSession;
use sodigy_span::SpanRange;

mod endec;

pub struct MirWarning {
    kind: MirWarningKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrorInfo,
}

impl SodigyError<MirWarningKind> for MirWarning {
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

    fn error_kind(&self) -> &MirWarningKind {
        &self.kind
    }

    fn is_warning(&self) -> bool {
        true
    }

    fn index(&self) -> u32 {
        9
    }

    fn get_stage(&self) -> Stage {
        Stage::Mir
    }
}

pub enum MirWarningKind {}

impl SodigyErrorKind for MirWarningKind {
    fn msg(&self, _: &mut InternSession) -> String {
        todo!()
    }

    fn help(&self, _: &mut InternSession) -> String {
        todo!()
    }

    fn index(&self) -> u32 {
        todo!()
    }
}
