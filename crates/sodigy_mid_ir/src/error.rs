use crate::func::LocalValue;
use smallvec::{SmallVec, smallvec};
use sodigy_error::{
    ExtraErrorInfo,
    RenderError,
    SodigyError,
    SodigyErrorKind,
    Stage,
};
use sodigy_high_ir::NameBindingType;
use sodigy_intern::{InternedString, InternSession};
use sodigy_span::SpanRange;

mod endec;

pub struct MirError {
    kind: MirErrorKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrorInfo,
}

impl MirError {
    pub fn recursive_local_value(local_value: &LocalValue, expr_span: SpanRange) -> Self {
        MirError {
            kind: MirErrorKind::RecursiveLocalValue { name: local_value.name.id(), name_binding_type: local_value.name_binding_type },
            spans: smallvec![*local_value.name.span(), expr_span],
            extra: ExtraErrorInfo::none(),
        }
    }
}

impl SodigyError<MirErrorKind> for MirError {
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

    fn error_kind(&self) -> &MirErrorKind {
        &self.kind
    }

    fn index(&self) -> u32 {
        8
    }

    fn get_stage(&self) -> Stage {
        Stage::Mir
    }
}

pub enum MirErrorKind {
    RecursiveLocalValue { name: InternedString, name_binding_type: NameBindingType },
}

impl SodigyErrorKind for MirErrorKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            MirErrorKind::RecursiveLocalValue { .. } => format!("a recursive local name binding"),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            MirErrorKind::RecursiveLocalValue { name, name_binding_type } => format!(
                "{} {} `{name}` is referencing it self.",
                name_binding_type.article(true),
                name_binding_type.render_error(),
            ),
        }
    }

    fn index(&self) -> u32 {
        match self {
            MirErrorKind::RecursiveLocalValue { .. } => 0,
        }
    }
}
