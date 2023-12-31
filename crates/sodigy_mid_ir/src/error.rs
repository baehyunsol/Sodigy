use smallvec::SmallVec;
use sodigy_error::{
    ExtraErrInfo,
    SodigyError,
    SodigyErrorKind,
};
use sodigy_intern::InternSession;
use sodigy_span::SpanRange;

pub struct MirError {
    kind: MirErrorKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrInfo,
}

impl SodigyError<MirErrorKind> for MirError {
    fn get_mut_error_info(&mut self) -> &mut ExtraErrInfo {
        &mut self.extra
    }

    fn get_error_info(&self) -> &ExtraErrInfo {
        &self.extra
    }

    fn get_first_span(&self) -> SpanRange {
        self.spans[0]
    }

    fn get_spans(&self) -> &[SpanRange] {
        &self.spans
    }

    fn err_kind(&self) -> &MirErrorKind {
        &self.kind
    }

    fn index(&self) -> u32 {
        7
    }
}

pub enum MirErrorKind {}

impl SodigyErrorKind for MirErrorKind {
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
